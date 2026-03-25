import { ApiClient } from './client';
import { Trade, Event } from './types';

export class TradesApi {
  constructor(private client: ApiClient) {}

  async getTrades(limit = 50, offset = 0): Promise<Trade[]> {
    return this.client.get(`/trades?limit=${limit}&offset=${offset}`);
  }

  async getTrade(id: string): Promise<Trade> {
    return this.client.get(`/trades/${id}`);
  }

  async createTrade(data: Partial<Trade>): Promise<Trade> {
    return this.client.post('/trades', data);
  }

  async updateTrade(id: string, data: Partial<Trade>): Promise<Trade> {
    return this.client.patch(`/trades/${id}`, data);
  }

  async deleteTrade(id: string): Promise<void> {
    return this.client.delete(`/trades/${id}`);
  }
}

export class EventsApi {
  constructor(private client: ApiClient) {}

  async getEvents(limit = 100, tradeId?: string): Promise<Event[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (tradeId) params.append('tradeId', tradeId);
    return this.client.get(`/events?${params}`);
  }

  async getEventsByTrade(tradeId: string): Promise<Event[]> {
    return this.client.get(`/events/trade/${tradeId}`);
  }

  async getEvent(id: string): Promise<Event> {
    return this.client.get(`/events/${id}`);
  }
}

export class BlockchainApi {
  constructor(private client: ApiClient) {}

  async fundTrade(tradeId: string, amount: string): Promise<{ txHash: string }> {
    return this.client.post(`/blockchain/fund`, { tradeId, amount });
  }

  async completeTrade(tradeId: string): Promise<{ txHash: string }> {
    return this.client.post(`/blockchain/complete`, { tradeId });
  }

  async resolvDispute(tradeId: string, resolution: string): Promise<{ txHash: string }> {
    return this.client.post(`/blockchain/resolve`, { tradeId, resolution });
  }

  async getTransactionStatus(txHash: string): Promise<{ status: string; confirmed: boolean }> {
    return this.client.get(`/blockchain/tx/${txHash}`);
  }
}
