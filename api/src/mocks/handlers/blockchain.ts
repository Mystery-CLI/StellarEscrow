import { rest } from 'msw';

export const blockchainHandlers = [
  rest.post('/api/blockchain/fund', (req, res, ctx) => {
    return res(ctx.json({ txHash: '0x' + Math.random().toString(16).slice(2) }));
  }),

  rest.post('/api/blockchain/complete', (req, res, ctx) => {
    return res(ctx.json({ txHash: '0x' + Math.random().toString(16).slice(2) }));
  }),

  rest.post('/api/blockchain/resolve', (req, res, ctx) => {
    return res(ctx.json({ txHash: '0x' + Math.random().toString(16).slice(2) }));
  }),

  rest.get('/api/blockchain/tx/:txHash', (req, res, ctx) => {
    return res(ctx.json({ status: 'confirmed', confirmed: true }));
  }),
];
