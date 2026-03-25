export interface Trade {
  id: string;
  seller: string;
  buyer: string;
  amount: string;
  status: 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled';
  arbitrator?: string;
  timestamp: string;
}

export interface Event {
  id: string;
  type: string;
  tradeId: string;
  timestamp: string;
  data: Record<string, any>;
}
