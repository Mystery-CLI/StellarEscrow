import { RootState } from './store';
import { Trade } from './types';

// Trades selectors
export const selectAllTrades = (state: RootState): Trade[] =>
  state.trades.allIds.map((id) => state.trades.byId[id]);

export const selectTradeById = (state: RootState, id: string): Trade | undefined =>
  state.trades.byId[id];

export const selectTradesByStatus = (state: RootState, status: string): Trade[] =>
  state.trades.allIds
    .map((id) => state.trades.byId[id])
    .filter((trade) => trade.status === status);

export const selectTradesLoading = (state: RootState): boolean => state.trades.loading;
export const selectTradesError = (state: RootState): string | null => state.trades.error;

// Events selectors
export const selectAllEvents = (state: RootState) =>
  state.events.allIds.map((id) => state.events.byId[id]);

export const selectEventsByTradeId = (state: RootState, tradeId: string) =>
  state.events.allIds
    .map((id) => state.events.byId[id])
    .filter((event) => event.tradeId === tradeId);

export const selectEventsLoading = (state: RootState): boolean => state.events.loading;
export const selectEventsError = (state: RootState): string | null => state.events.error;

// UI selectors
export const selectSelectedTradeId = (state: RootState): string | null =>
  state.ui.selectedTradeId;

export const selectFilters = (state: RootState) => state.ui.filters;
export const selectPagination = (state: RootState) => state.ui.pagination;

// Locale selectors
export const selectLocale = (state: RootState) => state.locale.locale;
export const selectIsRTL = (state: RootState) => state.locale.isRTL;
export const selectLocaleCurrency = (state: RootState) => state.locale.currency;
