import { rest } from 'msw';

const BASE = 'http://localhost:3000';

export const blockchainHandlers = [
  rest.post(`${BASE}/api/blockchain/fund`, (_req, res, ctx) => {
    return res(ctx.json({ txHash: '0xtx0001' }));
  }),

  rest.post(`${BASE}/api/blockchain/complete`, (_req, res, ctx) => {
    return res(ctx.json({ txHash: '0xtx0002' }));
  }),

  rest.post(`${BASE}/api/blockchain/resolve`, (_req, res, ctx) => {
    return res(ctx.json({ txHash: '0xtx0003' }));
  }),

  rest.get(`${BASE}/api/blockchain/tx/:txHash`, (_req, res, ctx) => {
    return res(ctx.json({ status: 'confirmed', confirmed: true }));
  }),
];