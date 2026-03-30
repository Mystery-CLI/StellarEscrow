import { rest } from 'msw';

const API_BASE_URL = 'http://localhost:3000/api';

export const blockchainHandlers = [
  rest.post(`${API_BASE_URL}/blockchain/fund`, (req, res, ctx) => {
    return res(ctx.json({ txHash: '0x' + Math.random().toString(16).slice(2) }));
  }),

  rest.post(`${API_BASE_URL}/blockchain/complete`, (req, res, ctx) => {
    return res(ctx.json({ txHash: '0x' + Math.random().toString(16).slice(2) }));
  }),

  rest.post(`${API_BASE_URL}/blockchain/resolve`, (req, res, ctx) => {
    return res(ctx.json({ txHash: '0x' + Math.random().toString(16).slice(2) }));
  }),

  rest.get(`${API_BASE_URL}/blockchain/tx/:txHash`, (req, res, ctx) => {
    return res(ctx.json({ status: 'confirmed', confirmed: true }));
  }),
];
