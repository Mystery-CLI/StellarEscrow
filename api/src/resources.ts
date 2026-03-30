import { ApiClient } from './client';
import { Event, EventCategory, Trade } from './models';

// =========================
// TRADES API
// =========================
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
    await this.client.delete(`/trades/${id}`);
    return undefined;
  }
}

// =========================
// EVENTS API
// =========================
export class EventsApi {
  constructor(private client: ApiClient) {}

  async getEvents(limit?: number, tradeId?: string): Promise<Event[]> {
    const params = new URLSearchParams();
    if (limit !== undefined) {
      params.append('limit', String(limit));
    }
    if (tradeId) {
      params.append('tradeId', tradeId);
    }
    return this.client.get(`/events?${params.toString()}`);
  }

  async getEventsByTrade(tradeId: string): Promise<Event[]> {
    return this.client.get(`/events/trade/${tradeId}`);
  }

  async getEventsByCategory(
    category: EventCategory,
    limit = 50
  ): Promise<Event[]> {
    const params = new URLSearchParams();
    params.append('category', category);
    params.append('limit', String(limit));
    return this.client.get(`/events?${params.toString()}`);
  }

  async getEvent(id: string): Promise<Event> {
    return this.client.get(`/events/${id}`);
  }
}

// =========================
// BLOCKCHAIN API
// =========================
export class BlockchainApi {
  constructor(private client: ApiClient) {}

  async fundTrade(
    tradeId: string,
    amount: string
  ): Promise<{ txHash: string }> {
    return this.client.post(`/blockchain/fund`, { tradeId, amount });
  }

  async completeTrade(
    tradeId: string
  ): Promise<{ txHash: string }> {
    return this.client.post(`/blockchain/complete`, { tradeId });
  }

  async resolveDispute(
    tradeId: string,
    resolution: string
  ): Promise<{ txHash: string }> {
    return this.client.post(`/blockchain/resolve`, {
      tradeId,
      resolution,
    });
  }


  async getTransactionStatus(txHash: string): Promise<{ status: string; confirmed: boolean }> {

  async resolvDispute(
    tradeId: string,
    resolution: string
  ): Promise<{ txHash: string }> {
    return this.resolveDispute(tradeId, resolution);
  }

  async getTransactionStatus(
    txHash: string
  ): Promise<{ status: string; confirmed: boolean }> {

    return this.client.get(`/blockchain/tx/${txHash}`);
  }
}