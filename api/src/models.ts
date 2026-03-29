export interface Trade {
  id: string;
  seller: string;
  buyer: string;
  amount: string;
  status: 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled';
  arbitrator?: string;
  timestamp: string;
}

/** High-level event categories matching the contract's topic layout. */
export type EventCategory =
  | 'trade'
  | 'arb'
  | 'fee'
  | 'tmpl'
  | 'sub'
  | 'gov'
  | 'sys'
  | 'ins'
  | 'oracle';

export interface Event {
  id: string;
  /** Canonical snake_case event name, e.g. "trade_created" */
  type: string;
  /** High-level category for client-side filtering */
  category: EventCategory;
  /** Schema version from the contract payload (v field) */
  schemaVersion: number;
  tradeId: string;
  timestamp: string;
  data: Record<string, unknown>;
}

/** Query parameters accepted by GET /events */
export interface EventFilterParams {
  limit?: number;
  offset?: number;
  event_type?: string;
  category?: EventCategory;
  trade_id?: string;
  from_ledger?: number;
  to_ledger?: number;
  from_time?: string;
  to_time?: string;
  contract_id?: string;
}
