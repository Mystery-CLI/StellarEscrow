import { BlockchainApi, EventsApi, TradesApi } from './resources';

function createMockClient() {
  return {
    get: jest.fn(),
    post: jest.fn(),
    patch: jest.fn(),
    delete: jest.fn(),
  };
}

describe('API endpoint mapping', () => {
  it('maps trade methods to the expected routes', async () => {
    const client = createMockClient();
    client.get.mockResolvedValue([]);
    client.post.mockResolvedValue({ id: '1' });
    client.patch.mockResolvedValue({ id: '1' });
    client.delete.mockResolvedValue(undefined);

    const trades = new TradesApi(client as never);

    await trades.getTrades(25, 10);
    await trades.getTrade('trade-1');
    await trades.createTrade({ seller: 'seller-1' });
    await trades.updateTrade('trade-1', { status: 'funded' });
    await trades.deleteTrade('trade-1');

    expect(client.get).toHaveBeenNthCalledWith(1, '/trades?limit=25&offset=10');
    expect(client.get).toHaveBeenNthCalledWith(2, '/trades/trade-1');
    expect(client.post).toHaveBeenCalledWith('/trades', { seller: 'seller-1' });
    expect(client.patch).toHaveBeenCalledWith('/trades/trade-1', { status: 'funded' });
    expect(client.delete).toHaveBeenCalledWith('/trades/trade-1');
  });

  it('maps event methods to the expected routes', async () => {
    const client = createMockClient();
    client.get.mockResolvedValue([]);

    const events = new EventsApi(client as never);

    await events.getEvents(20, 'trade-1');
    await events.getEventsByTrade('trade-1');
    await events.getEvent('event-1');

    expect(client.get).toHaveBeenNthCalledWith(1, '/events?limit=20&tradeId=trade-1');
    expect(client.get).toHaveBeenNthCalledWith(2, '/events/trade/trade-1');
    expect(client.get).toHaveBeenNthCalledWith(3, '/events/event-1');
  });

  it('maps blockchain methods to the expected routes', async () => {
    const client = createMockClient();
    client.get.mockResolvedValue({ status: 'confirmed', confirmed: true });
    client.post.mockResolvedValue({ txHash: '0xtx0001' });

    const blockchain = new BlockchainApi(client as never);

    await blockchain.fundTrade('trade-1', '100');
    await blockchain.completeTrade('trade-1');
    await blockchain.resolveDispute('trade-1', 'release_to_buyer');
    await blockchain.resolvDispute('trade-1', 'release_to_buyer');
    await blockchain.getTransactionStatus('0xtx0001');

    expect(client.post).toHaveBeenNthCalledWith(1, '/blockchain/fund', {
      tradeId: 'trade-1',
      amount: '100',
    });
    expect(client.post).toHaveBeenNthCalledWith(2, '/blockchain/complete', {
      tradeId: 'trade-1',
    });
    expect(client.post).toHaveBeenNthCalledWith(3, '/blockchain/resolve', {
      tradeId: 'trade-1',
      resolution: 'release_to_buyer',
    });
    expect(client.post).toHaveBeenNthCalledWith(4, '/blockchain/resolve', {
      tradeId: 'trade-1',
      resolution: 'release_to_buyer',
    });
    expect(client.get).toHaveBeenCalledWith('/blockchain/tx/0xtx0001');
  });
});
