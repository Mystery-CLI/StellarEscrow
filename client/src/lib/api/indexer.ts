import { apiFetch, type ClientConfig } from './client';
import type { Event, PagedResponse, TradeSearchResult, UserSearchResult, SearchSuggestion } from './types';

export function createIndexerApi(config: ClientConfig) {
  const get = <T>(path: string) => apiFetch<T>(config, path);
  const post = <T>(path: string, body: unknown) =>
    apiFetch<T>(config, path, { method: 'POST', body: JSON.stringify(body) });

  return {
    /** GET /events */
    getEvents(params?: {
      limit?: number;
      offset?: number;
      event_type?: string;
      trade_id?: number;
    }): Promise<PagedResponse<Event>> {
      const qs = new URLSearchParams(
        Object.entries(params ?? {})
          .filter(([, v]) => v !== undefined)
          .map(([k, v]) => [k, String(v)])
      ).toString();
      return get(`/events${qs ? `?${qs}` : ''}`);
    },

    /** GET /events/:id */
    getEvent(id: string): Promise<Event> {
      return get(`/events/${id}`);
    },

    /** GET /events/trade/:trade_id */
    getTradeEvents(tradeId: number): Promise<Event[]> {
      return get(`/events/trade/${tradeId}`);
    },

    /** GET /search/trades */
    searchTrades(query: string, filters?: { status?: string; minAmount?: number; maxAmount?: number }): Promise<PagedResponse<TradeSearchResult>> {
      const qs = new URLSearchParams({ q: query, ...Object.fromEntries(Object.entries(filters ?? {}).filter(([,v]) => v !== undefined).map(([k,v]) => [k, String(v)])) }).toString();
      return get(`/search/trades?${qs}`);
    },

    /** GET /search/users */
    searchUsers(query: string): Promise<PagedResponse<UserSearchResult>> {
      return get(`/search/users?q=${encodeURIComponent(query)}`);
    },

    /** GET /search/suggestions */
    getSearchSuggestions(query: string): Promise<SearchSuggestion[]> {
      return get(`/search/suggestions?q=${encodeURIComponent(query)}`);
    },

    /** POST /events/replay */
    replayEvents(fromLedger: number, toLedger: number): Promise<{ replayed: number }> {
      return post('/events/replay', { from_ledger: fromLedger, to_ledger: toLedger });
    },
  };
}

export type IndexerApi = ReturnType<typeof createIndexerApi>;
