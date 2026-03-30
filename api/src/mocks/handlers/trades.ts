import { rest } from 'msw';
import { Trade } from '../../models';
import { mockTrades } from '../data';

const API_BASE_URL = 'http://localhost:3000/api';

const BASE = 'http://localhost:3000';

export const tradeHandlers = [

  rest.get(`${API_BASE_URL}/trades`, (req, res, ctx) => {

  rest.get(`${BASE}/api/trades`, (req, res, ctx) => {

    const limit = req.url.searchParams.get('limit') || '50';
    return res(ctx.json(mockTrades.slice(0, parseInt(limit))));
  }),


  rest.get(`${API_BASE_URL}/trades/:id`, (req, res, ctx) => {

  rest.get(`${BASE}/api/trades/:id`, (req, res, ctx) => {

    const trade = mockTrades.find((t) => t.id === req.params.id);
    return trade
      ? res(ctx.json(trade))
      : res(ctx.status(404), ctx.json({ error: 'Not found' }));
  }),


  rest.post(`${API_BASE_URL}/trades`, (req, res, ctx) => {
    const body = (req.body ?? {}) as Partial<Trade>;
    const newTrade: Trade = { id: String(mockTrades.length + 1), ...body } as Trade;
    mockTrades.push(newTrade);
    return res(ctx.status(201), ctx.json(newTrade));
  }),

  rest.patch(`${API_BASE_URL}/trades/:id`, (req, res, ctx) => {

  rest.post(`${BASE}/api/trades`, async (req, res, ctx) => {
    const body = await req.json();
    const newTrade = { id: String(mockTrades.length + 1), ...body };
    mockTrades.push(newTrade as any);
    return res(ctx.status(201), ctx.json(newTrade));
  }),

  rest.patch(`${BASE}/api/trades/:id`, async (req, res, ctx) => {

    const trade = mockTrades.find((t) => t.id === req.params.id);
    if (!trade) return res(ctx.status(404), ctx.json({ error: 'Not found' }));
    const body = await req.json();
    Object.assign(trade, body);
    return res(ctx.json(trade));
  }),


  rest.delete(`${API_BASE_URL}/trades/:id`, (req, res, ctx) => {

  rest.delete(`${BASE}/api/trades/:id`, (req, res, ctx) => {

    const index = mockTrades.findIndex((t) => t.id === req.params.id);
    if (index === -1) return res(ctx.status(404), ctx.json({ error: 'Not found' }));
    mockTrades.splice(index, 1);
    return res(ctx.status(204));
  }),
];