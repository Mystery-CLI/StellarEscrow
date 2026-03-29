import { store } from '../store';
import { addTrade, updateTrade, setTrades } from '../slices/tradesSlice';
import { addEvent, setEvents } from '../slices/eventsSlice';
import { Trade, Event } from '../types';

describe('Redux Store', () => {
  it('should initialize with empty state', () => {
    const state = store.getState();
    expect(state.trades.byId).toEqual({});
    expect(state.trades.allIds).toEqual([]);
    expect(state.events.byId).toEqual({});
    expect(state.events.allIds).toEqual([]);
  });

  it('should add a trade', () => {
    const trade: Trade = {
      id: '1',
      seller: 'G123',
      buyer: 'G456',
      amount: '100',
      status: 'created',
      timestamp: '2024-03-25',
    };
    store.dispatch(addTrade(trade));
    const state = store.getState();
    expect(state.trades.byId['1']).toEqual(trade);
    expect(state.trades.allIds).toContain('1');
  });

  it('should update a trade', () => {
    const trade: Trade = {
      id: '2',
      seller: 'G123',
      buyer: 'G456',
      amount: '100',
      status: 'created',
      timestamp: '2024-03-25',
    };
    store.dispatch(addTrade(trade));
    store.dispatch(updateTrade({ id: '2', status: 'funded' }));
    const state = store.getState();
    expect(state.trades.byId['2'].status).toBe('funded');
  });

  it('should add an event', () => {
    const event: Event = {
      id: '1',
      type: 'trade_created',
      tradeId: '1',
      timestamp: '2024-03-25',
      data: {},
    };
    store.dispatch(addEvent(event));
    const state = store.getState();
    expect(state.events.byId['1']).toEqual(event);
  });

  it('should normalize trades state', () => {
    const trades: Trade[] = [
      { id: '1', seller: 'G1', buyer: 'G2', amount: '100', status: 'created', timestamp: '2024-03-25' },
      { id: '2', seller: 'G3', buyer: 'G4', amount: '200', status: 'funded', timestamp: '2024-03-25' },
    ];
    store.dispatch(setTrades(trades));
    const state = store.getState();
    expect(Object.keys(state.trades.byId).length).toBe(2);
    expect(state.trades.allIds.length).toBe(2);
  });
});
