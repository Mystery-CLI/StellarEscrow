import { API_ENDPOINT_CONTRACTS } from './contracts';
import { createApi } from './index';

describe('API contract', () => {
  const api = createApi('http://localhost:3000');

  const invokers: Record<string, () => Promise<unknown>> = {
    listTrades: () => api.trades.getTrades(10, 0),
    getTrade: () => api.trades.getTrade('1'),
    createTrade: () =>
      api.trades.createTrade({
        seller: 'GSeller',
        buyer: 'GBuyer',
        amount: '100',
        status: 'created',
        timestamp: '2024-03-25T10:30:00Z',
      }),
    updateTrade: () => api.trades.updateTrade('1', { status: 'funded' }),
    deleteTrade: () => api.trades.deleteTrade('1'),
    listEvents: () => api.events.getEvents(100),
    getEventsByTrade: () => api.events.getEventsByTrade('1'),
    getEvent: () => api.events.getEvent('1'),
    fundTrade: () => api.blockchain.fundTrade('1', '100'),
    completeTrade: () => api.blockchain.completeTrade('1'),
    resolveDispute: () => api.blockchain.resolveDispute('1', 'release_to_buyer'),
    getTransactionStatus: () => api.blockchain.getTransactionStatus('0xtx0001'),
  };

  it('keeps the contract registry free of duplicate method/path pairs', () => {
    const keys = API_ENDPOINT_CONTRACTS.map((contract) => `${contract.method} ${contract.path}`);
    expect(new Set(keys).size).toBe(keys.length);
  });

  it.each(API_ENDPOINT_CONTRACTS)(
    '$id returns a response that matches the documented contract',
    async (contract) => {
      const response = await invokers[contract.id]();
      expect(contract.validateResponse(response)).toBe(true);
    }
  );
});
