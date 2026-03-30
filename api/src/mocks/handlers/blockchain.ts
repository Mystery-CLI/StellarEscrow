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