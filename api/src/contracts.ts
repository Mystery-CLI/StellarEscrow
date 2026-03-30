import { Event, Trade } from './models';

export type HttpMethod = 'GET' | 'POST' | 'PATCH' | 'DELETE';

export interface EndpointContract<TResponse = unknown> {
  id: string;
  area: 'trades' | 'events' | 'blockchain';
  method: HttpMethod;
  path: string;
  clientMethod: string;
  readmeSection: string;
  validateResponse: (value: unknown) => value is TResponse;
}

const tradeStatuses = new Set<Trade['status']>([
  'created',
  'funded',
  'completed',
  'disputed',
  'cancelled',
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

export function isTrade(value: unknown): value is Trade {
  return (
    isRecord(value) &&
    typeof value.id === 'string' &&
    typeof value.seller === 'string' &&
    typeof value.buyer === 'string' &&
    typeof value.amount === 'string' &&
    typeof value.timestamp === 'string' &&
    typeof value.status === 'string' &&
    tradeStatuses.has(value.status as Trade['status']) &&
    (value.arbitrator === undefined || typeof value.arbitrator === 'string')
  );
}

const EVENT_CATEGORIES = new Set<string>([
  'trade', 'arb', 'fee', 'tmpl', 'sub', 'gov', 'sys', 'ins', 'oracle',
]);

export function isEvent(value: unknown): value is Event {
  return (
    isRecord(value) &&
    typeof value.id === 'string' &&
    typeof value.type === 'string' &&
    typeof value.category === 'string' &&
    EVENT_CATEGORIES.has(value.category) &&
    typeof value.schemaVersion === 'number' &&
    typeof value.tradeId === 'string' &&
    typeof value.timestamp === 'string' &&
    isRecord(value.data)
  );
}

export function isTradeList(value: unknown): value is Trade[] {
  return Array.isArray(value) && value.every(isTrade);
}

export function isEventList(value: unknown): value is Event[] {
  return Array.isArray(value) && value.every(isEvent);
}

export function isTxHashResponse(value: unknown): value is { txHash: string } {
  return isRecord(value) && typeof value.txHash === 'string' && value.txHash.length > 0;
}

export function isTransactionStatusResponse(
  value: unknown
): value is { status: string; confirmed: boolean } {
  return (
    isRecord(value) &&
    typeof value.status === 'string' &&
    typeof value.confirmed === 'boolean'
  );
}

export const API_ENDPOINT_CONTRACTS: EndpointContract[] = [
  {
    id: 'listTrades',
    area: 'trades',
    method: 'GET',
    path: '/api/trades',
    clientMethod: 'trades.getTrades(limit, offset)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTradeList,
  },
  {
    id: 'getTrade',
    area: 'trades',
    method: 'GET',
    path: '/api/trades/:id',
    clientMethod: 'trades.getTrade(id)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTrade,
  },
  {
    id: 'createTrade',
    area: 'trades',
    method: 'POST',
    path: '/api/trades',
    clientMethod: 'trades.createTrade(data)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTrade,
  },
  {
    id: 'updateTrade',
    area: 'trades',
    method: 'PATCH',
    path: '/api/trades/:id',
    clientMethod: 'trades.updateTrade(id, data)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTrade,
  },
  {
    id: 'deleteTrade',
    area: 'trades',
    method: 'DELETE',
    path: '/api/trades/:id',
    clientMethod: 'trades.deleteTrade(id)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: (value: unknown): value is void => value === undefined || value === '',
  },
  {
    id: 'listEvents',
    area: 'events',
    method: 'GET',
    path: '/api/events',
    clientMethod: 'events.getEvents(limit, tradeId?)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isEventList,
  },
  {
    id: 'getEventsByTrade',
    area: 'events',
    method: 'GET',
    path: '/api/events/trade/:tradeId',
    clientMethod: 'events.getEventsByTrade(tradeId)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isEventList,
  },
  {
    id: 'getEvent',
    area: 'events',
    method: 'GET',
    path: '/api/events/:id',
    clientMethod: 'events.getEvent(id)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isEvent,
  },
  {
    id: 'fundTrade',
    area: 'blockchain',
    method: 'POST',
    path: '/api/blockchain/fund',
    clientMethod: 'blockchain.fundTrade(tradeId, amount)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTxHashResponse,
  },
  {
    id: 'completeTrade',
    area: 'blockchain',
    method: 'POST',
    path: '/api/blockchain/complete',
    clientMethod: 'blockchain.completeTrade(tradeId)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTxHashResponse,
  },
  {
    id: 'resolveDispute',
    area: 'blockchain',
    method: 'POST',
    path: '/api/blockchain/resolve',
    clientMethod: 'blockchain.resolveDispute(tradeId, resolution)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTxHashResponse,
  },
  {
    id: 'getTransactionStatus',
    area: 'blockchain',
    method: 'GET',
    path: '/api/blockchain/tx/:txHash',
    clientMethod: 'blockchain.getTransactionStatus(txHash)',
    readmeSection: 'Endpoint Matrix',
    validateResponse: isTransactionStatusResponse,
  },
];
