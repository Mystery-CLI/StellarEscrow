import { tradeHandlers } from './mocks/handlers/trades';
import { eventHandlers } from './mocks/handlers/events';
import { blockchainHandlers } from './mocks/handlers/blockchain';
import { integrationHandlers } from './mocks/handlers/integration';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { Event, Trade } from './models';

const API_BASE_URL = 'http://localhost:3000/api';

const initialTrades: Trade[] = [
  {
    id: '1',
    seller: 'GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE',
    buyer: 'GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD',
    amount: '100.50',
    status: 'completed',
    timestamp: '2024-03-25T10:30:00Z',
  },
];

const initialEvents: Event[] = [
  {
    id: '1',
    type: 'trade_created',
    category: 'trade',
    schemaVersion: 1,
    tradeId: '1',
    timestamp: '2024-03-25T10:30:00Z',
    data: {},
  },
];

let mockTrades: Trade[] = [];
let mockEvents: Event[] = [];
let txCounter = 0;

function cloneTrades(): Trade[] {
  return initialTrades.map((trade) => ({ ...trade }));
}

function cloneEvents(): Event[] {
  return initialEvents.map((event) => ({ ...event, data: { ...event.data } }));
}

function nextTxHash(): string {
  txCounter += 1;
  return `0xtx${txCounter.toString().padStart(4, '0')}`;
}

export function resetMockData() {
  mockTrades = cloneTrades();
  mockEvents = cloneEvents();
  txCounter = 0;
}

resetMockData();

export const handlers = [
  ...tradeHandlers,
  ...eventHandlers,
  ...blockchainHandlers,
  ...integrationHandlers,
];

export const server = setupServer(...handlers);
