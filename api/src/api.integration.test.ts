import { rest } from 'msw';
import { createApi } from './index';
import { server } from './mocks';

describe('API integration', () => {
  it('runs the documented happy path against mocked endpoints', async () => {
    const api = createApi('http://localhost:3000');

    await expect(api.trades.getTrades(10, 0)).resolves.toHaveLength(1);
    await expect(api.trades.getTrade('1')).resolves.toMatchObject({ id: '1' });
    await expect(
      api.trades.createTrade({
        seller: 'GSeller',
        buyer: 'GBuyer',
        amount: '100',
        status: 'created',
        timestamp: '2024-03-25T10:30:00Z',
      })
    ).resolves.toMatchObject({ id: '2', amount: '100' });
    await expect(api.trades.updateTrade('1', { status: 'funded' })).resolves.toMatchObject({
      id: '1',
      status: 'funded',
    });
    await expect(api.trades.deleteTrade('1')).resolves.toBeUndefined();

    await expect(api.events.getEvents(100)).resolves.toHaveLength(1);
    await expect(api.events.getEvents(100, '1')).resolves.toHaveLength(1);
    await expect(api.events.getEventsByTrade('1')).resolves.toHaveLength(1);
    await expect(api.events.getEvent('1')).resolves.toMatchObject({ id: '1', tradeId: '1' });

    await expect(api.blockchain.fundTrade('1', '100')).resolves.toMatchObject({
      txHash: '0xtx0001',
    });
    await expect(api.blockchain.completeTrade('1')).resolves.toMatchObject({
      txHash: '0xtx0002',
    });
    await expect(api.blockchain.resolveDispute('1', 'release_to_buyer')).resolves.toMatchObject({
      txHash: '0xtx0003',
    });
    await expect(api.blockchain.getTransactionStatus('0xtx0003')).resolves.toEqual({
      status: 'confirmed',
      confirmed: true,
    });
  });

  it('sends auth headers through request interceptors', async () => {
    const api = createApi('http://localhost:3000');
    let authorizationHeader: string | null = null;

    server.use(
      rest.get('/api/trades', (req, res, ctx) => {
        authorizationHeader = req.headers.get('authorization');
        return res(ctx.json([]));
      })
    );

    api.addAuthToken('secure-token');
    await api.trades.getTrades();

    expect(authorizationHeader).toBe('Bearer secure-token');
  });

  it('invokes registered error handlers and rethrows the original failure', async () => {
    const api = createApi('http://localhost:3000');
    const handler = jest.fn();

    server.use(
      rest.get('/api/trades', (_req, res, ctx) =>
        res(ctx.status(500), ctx.json({ error: 'temporary outage' }))
      )
    );

    api.addErrorHandler(handler);

    await expect(api.trades.getTrades()).rejects.toMatchObject({
      status: 500,
      details: { error: 'temporary outage' },
    });
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it('retries transient failures before returning a successful response', async () => {
    const api = createApi('http://localhost:3000');
    let attempts = 0;

    server.use(
      rest.get('/api/trades', (_req, res, ctx) => {
        attempts += 1;
        if (attempts === 1) {
          return res(ctx.status(503), ctx.json({ error: 'retry me' }));
        }
        return res(ctx.json([{ id: '1', seller: 'A', buyer: 'B', amount: '1', status: 'created', timestamp: '2024-03-25T10:30:00Z' }]));
      })
    );

    await expect(api.trades.getTrades()).resolves.toHaveLength(1);
    expect(attempts).toBe(2);
  });
});
