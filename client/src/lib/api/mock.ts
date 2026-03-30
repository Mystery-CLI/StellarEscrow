import type { Event, PagedResponse, TradeSearchResult, UserSearchResult, SearchSuggestion } from './types';
import type { IndexerApi } from './indexer';

const MOCK_EVENTS: Event[] = [
  {
    id: 'mock-1',
    event_type: 'trade_created',
    contract_id: 'CDMOCK000',
    ledger: 1000,
    transaction_hash: 'abc123',
    timestamp: new Date().toISOString(),
    data: { trade_id: 1, seller: 'GSELLER111AAA', buyer: 'GBUYER222BBB', amount: 1000000 },
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

const MOCK_TRADES: TradeSearchResult[] = [
  { trade_id: 1, seller: 'GSELLER111AAA', buyer: 'GBUYER222BBB', amount: 1000000, status: 'funded', created_at: new Date().toISOString() },
  { trade_id: 2, seller: 'GSELLER333CCC', buyer: 'GBUYER444DDD', amount: 5000000, status: 'created', created_at: new Date().toISOString() },
  { trade_id: 3, seller: 'GSELLER555EEE', buyer: 'GBUYER666FFF', amount: 2500000, status: 'completed', created_at: new Date().toISOString() },
  { trade_id: 4, seller: 'GARB777GGG', buyer: 'GBUYER888HHH', amount: 750000, status: 'disputed', created_at: new Date().toISOString() },
];

const MOCK_USERS: UserSearchResult[] = [
  { address: 'GSELLER111AAA', role: 'seller', trade_count: 12 },
  { address: 'GBUYER222BBB', role: 'buyer', trade_count: 5 },
  { address: 'GARB777GGG', role: 'arbitrator', trade_count: 30 },
  { address: 'GARB999ZZZ', role: 'arbitrator', trade_count: 18 },
];

const delay = (ms = 200) => new Promise((r) => setTimeout(r, ms));

const match = (obj: unknown, q: string) => JSON.stringify(obj).toLowerCase().includes(q.toLowerCase());

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

  async searchTrades(query, filters) {
    await delay();
    let items = MOCK_TRADES.filter((t) => !query || match(t, query));
    if (filters?.status) items = items.filter((t) => t.status === filters.status);
    if (filters?.minAmount !== undefined) items = items.filter((t) => t.amount >= filters.minAmount!);
    if (filters?.maxAmount !== undefined) items = items.filter((t) => t.amount <= filters.maxAmount!);
    return { items, total: items.length, limit: 20, offset: 0 };
  },

  async searchUsers(query) {
    await delay();
    const items = MOCK_USERS.filter((u) => !query || match(u, query));
    return { items, total: items.length, limit: 20, offset: 0 };
  },

  async getSearchSuggestions(query) {
    await delay(100);
    if (!query) return [];
    const suggestions: SearchSuggestion[] = [];
    MOCK_TRADES.filter((t) => match(t, query)).slice(0, 3).forEach((t) =>
      suggestions.push({ label: `Trade #${t.trade_id} — ${t.status}`, type: 'trade', value: String(t.trade_id) })
    );
    MOCK_USERS.filter((u) => match(u, query)).slice(0, 3).forEach((u) =>
      suggestions.push({ label: `${u.address.slice(0, 12)}… (${u.role})`, type: u.role === 'arbitrator' ? 'arbitrator' : 'user', value: u.address })
    );
    return suggestions;
  },

  async replayEvents(fromLedger, toLedger) {
    await delay(500);
    return { replayed: toLedger - fromLedger };
  },
};
