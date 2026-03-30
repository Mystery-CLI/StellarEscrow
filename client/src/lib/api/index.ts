import { createIndexerApi } from './indexer';
import { mockIndexerApi } from './mock';
import type { ClientConfig } from './client';

const USE_MOCK = import.meta.env.VITE_API_MOCK === 'true';
const BASE_URL = import.meta.env.VITE_INDEXER_URL ?? 'http://localhost:3000';

const config: ClientConfig = {
  baseUrl: BASE_URL,
  retries: 3,
  onRequest(url, init) {
    // Attach auth token if present (e.g. from localStorage)
    const token = typeof localStorage !== 'undefined' ? localStorage.getItem('auth_token') : null;
    if (token) {
      init.headers = { ...init.headers as Record<string, string>, Authorization: `Bearer ${token}` };
    }
    return init;
  },
  onResponse(url, res) {
    if (import.meta.env.DEV) {
      console.debug(`[api] ${res.status} ${url}`);
    }
  },
};

export const indexerApi = USE_MOCK ? mockIndexerApi : createIndexerApi(config);

export { ApiRequestError } from './types';
export type { Event, PagedResponse, TradeSearchResult, UserSearchResult, SearchSuggestion } from './types';
export type { IndexerApi } from './indexer';
