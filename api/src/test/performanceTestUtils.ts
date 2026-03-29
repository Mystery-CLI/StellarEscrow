import { Event, Trade } from '../models';

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function createTradeFixture(overrides: Partial<Trade> = {}): Trade {
  return {
    id: '1',
    seller: 'A',
    buyer: 'B',
    amount: '100',
    status: 'created',
    timestamp: '2024-03-25T10:30:00Z',
    ...overrides,
  };
}

export function createEventFixture(overrides: Partial<Event> = {}): Event {
  return {
    id: '1',
    type: 'trade_created',
    tradeId: '1',
    timestamp: '2024-03-25T10:30:00Z',
    data: {},
    ...overrides,
  };
}
