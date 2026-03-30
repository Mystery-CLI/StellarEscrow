import { writable, derived, get } from 'svelte/store';
import { indexerApi } from './api';
import type { TradeSearchResult, UserSearchResult, SearchSuggestion } from './api';

const HISTORY_KEY = 'stellar-escrow-search-history';
const MAX_HISTORY = 10;

function loadHistory(): string[] {
  if (typeof localStorage === 'undefined') return [];
  try { return JSON.parse(localStorage.getItem(HISTORY_KEY) ?? '[]'); } catch { return []; }
}

function saveHistory(h: string[]) {
  if (typeof localStorage !== 'undefined') localStorage.setItem(HISTORY_KEY, JSON.stringify(h));
}

export const searchQuery = writable('');
export const statusFilter = writable('');
export const minAmount = writable<number | undefined>(undefined);
export const maxAmount = writable<number | undefined>(undefined);

export const trades = writable<TradeSearchResult[]>([]);
export const users = writable<UserSearchResult[]>([]);
export const suggestions = writable<SearchSuggestion[]>([]);
export const searchHistory = writable<string[]>(loadHistory());
export const loading = writable(false);
export const error = writable('');

export function addToHistory(q: string) {
  if (!q.trim()) return;
  searchHistory.update((h) => {
    const next = [q, ...h.filter((x) => x !== q)].slice(0, MAX_HISTORY);
    saveHistory(next);
    return next;
  });
}

export function clearHistory() {
  searchHistory.set([]);
  saveHistory([]);
}

let suggestTimer: ReturnType<typeof setTimeout>;

export async function fetchSuggestions(q: string) {
  clearTimeout(suggestTimer);
  if (!q.trim()) { suggestions.set([]); return; }
  suggestTimer = setTimeout(async () => {
    suggestions.set(await indexerApi.getSearchSuggestions(q));
  }, 200);
}

export async function runSearch() {
  const q = get(searchQuery);
  const status = get(statusFilter);
  const min = get(minAmount);
  const max = get(maxAmount);

  loading.set(true);
  error.set('');
  suggestions.set([]);
  addToHistory(q);

  try {
    const [tradeRes, userRes] = await Promise.all([
      indexerApi.searchTrades(q, { status: status || undefined, minAmount: min, maxAmount: max }),
      indexerApi.searchUsers(q),
    ]);
    trades.set(tradeRes.items);
    users.set(userRes.items);
  } catch (e: any) {
    error.set(e.message ?? 'Search failed');
  } finally {
    loading.set(false);
  }
}
