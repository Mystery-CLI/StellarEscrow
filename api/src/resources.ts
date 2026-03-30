import { ApiClient } from './client';
import { Event, EventCategory, EventFilterParams, Trade } from './models';

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

  async getEvents(filters: EventFilterParams = {}): Promise<Event[]> {
    const params = new URLSearchParams();
    if (filters.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters.offset !== undefined) params.set('offset', filters.offset.toString());
    if (filters.event_type) params.set('event_type', filters.event_type);
    if (filters.category) params.set('category', filters.category);
    if (filters.trade_id) params.set('trade_id', filters.trade_id);
    if (filters.from_ledger !== undefined) params.set('from_ledger', filters.from_ledger.toString());
    if (filters.to_ledger !== undefined) params.set('to_ledger', filters.to_ledger.toString());
    if (filters.from_time) params.set('from_time', filters.from_time);
    if (filters.to_time) params.set('to_time', filters.to_time);
    if (filters.contract_id) params.set('contract_id', filters.contract_id);
    return this.client.get(`/events?${params}`);
  }

  async getEventsByTrade(tradeId: string): Promise<Event[]> {
    return this.client.get(`/events/trade/${tradeId}`);
  }

  async getEventsByCategory(category: EventCategory, limit = 50): Promise<Event[]> {
    return this.getEvents({ category, limit });
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

  async resolveDispute(tradeId: string, resolution: string): Promise<{ txHash: string }> {
    return this.client.post(`/blockchain/resolve`, { tradeId, resolution });
  }

  async getTransactionStatus(txHash: string): Promise<{ status: string; confirmed: boolean }> {
    return this.client.get(`/blockchain/tx/${txHash}`);
  }
}
