import { RootState } from './store';
import { Trade, Event } from './types';
import { tradesAdapter } from './slices/tradesSlice';
import { eventsAdapter } from './slices/eventsSlice';

const tradeSelectors = tradesAdapter.getSelectors<RootState>((state) => state.trades);
const eventSelectors = eventsAdapter.getSelectors<RootState>((state) => state.events);

// Trades selectors
export const selectAllTrades = tradeSelectors.selectAll;
export const selectTradeById = tradeSelectors.selectById;

export const selectTradesByStatus = (state: RootState, status: string): Trade[] =>
  tradeSelectors.selectAll(state).filter((trade) => trade.status === status);

export const selectTradesLoading = (state: RootState): boolean => state.trades.loading;
export const selectTradesError = (state: RootState): string | null => state.trades.error;

// Events selectors
export const selectAllEvents = eventSelectors.selectAll;
export const selectEventsByTradeId = (state: RootState, tradeId: string): Event[] =>
  eventSelectors.selectAll(state).filter((event) => event.tradeId === tradeId);

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
