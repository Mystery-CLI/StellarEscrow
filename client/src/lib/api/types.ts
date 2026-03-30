export interface Event {
  id: string;
  event_type: string;
  contract_id: string;
  ledger: number;
  transaction_hash: string;
  timestamp: string;
  data: Record<string, unknown>;
  created_at: string;
}

export interface PagedResponse<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
}

export interface TradeSearchResult {
  trade_id: number;
  seller: string;
  buyer: string;
  amount: number;
  status: string;
  created_at: string;
}

export interface UserSearchResult {
  address: string;
  role: 'buyer' | 'seller' | 'arbitrator';
  trade_count: number;
}

export interface SearchSuggestion {
  label: string;
  type: 'trade' | 'user' | 'arbitrator';
  value: string;
}

export interface ApiError {
  code: string;
  message: string;
  status: number;
}

export class ApiRequestError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string
  ) {
    super(message);
    this.name = 'ApiRequestError';
  }
}
