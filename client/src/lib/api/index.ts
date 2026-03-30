import { createIndexerApi } from './indexer';
import { mockIndexerApi } from './mock';
import type { ClientConfig } from './client';
import { env } from '$lib/env';

const config: ClientConfig = {
  baseUrl: env.indexerUrl,
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

export const indexerApi = env.apiMock ? mockIndexerApi : createIndexerApi(config);

export { ApiRequestError } from './types';
export { invalidateCache } from './client';
export type { Event, PagedResponse, TradeSearchResult, UserSearchResult, SearchSuggestion } from './types';
export type { IndexerApi } from './indexer';
