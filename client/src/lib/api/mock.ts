import type { Event, PagedResponse, TradeSearchResult } from './types';
import type { IndexerApi } from './indexer';

const MOCK_EVENTS: Event[] = [
  {
    id: 'mock-1',
    event_type: 'trade_created',
    contract_id: 'CDMOCK000',
    ledger: 1000,
    transaction_hash: 'abc123',
    timestamp: new Date().toISOString(),
    data: { trade_id: 1, seller: 'GSELLER', buyer: 'GBUYER', amount: 1000000 },
    created_at: new Date().toISOString(),
  },
  {
    id: 'mock-2',
    event_type: 'trade_funded',
    contract_id: 'CDMOCK000',
    ledger: 1010,
    transaction_hash: 'def456',
    timestamp: new Date().toISOString(),
    data: { trade_id: 1 },
    created_at: new Date().toISOString(),
  },
];

const delay = (ms = 200) => new Promise((r) => setTimeout(r, ms));

export const mockIndexerApi: IndexerApi = {
  async getEvents(params) {
    await delay();
    const limit = params?.limit ?? 20;
    const offset = params?.offset ?? 0;
    let items = MOCK_EVENTS;
    if (params?.event_type) items = items.filter((e) => e.event_type === params.event_type);
    if (params?.trade_id) items = items.filter((e) => (e.data as any).trade_id === params.trade_id);
    return { items: items.slice(offset, offset + limit), total: items.length, limit, offset };
  },

  async getEvent(id) {
    await delay();
    const ev = MOCK_EVENTS.find((e) => e.id === id);
    if (!ev) throw new Error(`Event ${id} not found`);
    return ev;
  },

  async getTradeEvents(tradeId) {
    await delay();
    return MOCK_EVENTS.filter((e) => (e.data as any).trade_id === tradeId);
  },

  async searchTrades(query) {
    await delay();
    const results: TradeSearchResult[] = [
      { trade_id: 1, seller: 'GSELLER', buyer: 'GBUYER', amount: 1000000, status: 'funded', created_at: new Date().toISOString() },
    ].filter((t) => JSON.stringify(t).toLowerCase().includes(query.toLowerCase()));
    return { items: results, total: results.length, limit: 20, offset: 0 };
  },

  async replayEvents(fromLedger, toLedger) {
    await delay(500);
    return { replayed: toLedger - fromLedger };
  },
};
