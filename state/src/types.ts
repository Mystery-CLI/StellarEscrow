import { LocaleState } from './slices/localeSlice';

// Trade state types
export interface Trade {
  id: string;
  seller: string;
  buyer: string;
  amount: string;
  status: 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled';
  arbitrator?: string;
  timestamp: string;
}

export interface TradesState {
  byId: Record<string, Trade>;
  allIds: string[];
  loading: boolean;
  error: string | null;
}

// Event state types
export interface Event {
  id: string;
  type: string;
  tradeId: string;
  timestamp: string;
  data: Record<string, any>;
}

export interface EventsState {
  byId: Record<string, Event>;
  allIds: string[];
  loading: boolean;
  error: string | null;
}

// UI state types
export interface UIState {
  selectedTradeId: string | null;
  filters: {
    status?: string;
    tradeId?: string;
  };
  pagination: {
    page: number;
    pageSize: number;
  };
}

// Root state
export interface RootState {
  trades: TradesState;
  events: EventsState;
  ui: UIState;
  locale: LocaleState;
}
