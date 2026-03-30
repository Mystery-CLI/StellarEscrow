import { rest } from 'msw';
import { mockEvents } from '../data';

const BASE = 'http://localhost:3000';

export const eventHandlers = [
  rest.get(`${BASE}/api/events`, (req, res, ctx) => {
    const limit = req.url.searchParams.get('limit') || '100';
    return res(ctx.json(mockEvents.slice(0, parseInt(limit))));
  }),

  rest.get(`${BASE}/api/events/trade/:tradeId`, (req, res, ctx) => {
    const events = mockEvents.filter((e) => e.tradeId === req.params.tradeId);
    return res(ctx.json(events));
  }),

  rest.get(`${BASE}/api/events/:id`, (req, res, ctx) => {
    const event = mockEvents.find((e) => e.id === req.params.id);
    return event
      ? res(ctx.json(event))
      : res(ctx.status(404), ctx.json({ error: 'Not found' }));
  }),
];