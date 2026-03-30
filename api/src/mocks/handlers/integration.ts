import { rest } from 'msw';

const THIRD_PARTY_BASE = 'https://api.example-integration.com';

export const integrationHandlers = [
  // Health-check endpoint
  rest.get(`${THIRD_PARTY_BASE}/health`, (_req, res, ctx) =>
    res(ctx.json({ status: 'ok' }))
  ),

  // Generic resource endpoint — echo the request back
  rest.get(`${THIRD_PARTY_BASE}/resource`, (_req, res, ctx) =>
    res(ctx.json({ items: [{ id: 'r1', value: 'test' }] }))
  ),

  rest.post(`${THIRD_PARTY_BASE}/resource`, (req, res, ctx) =>
    res(ctx.status(201), ctx.json({ id: 'r-new', ...(req.body as object) }))
  ),

  rest.patch(`${THIRD_PARTY_BASE}/resource/:id`, (req, res, ctx) =>
    res(ctx.json({ id: req.params.id, ...(req.body as object) }))
  ),

  rest.delete(`${THIRD_PARTY_BASE}/resource/:id`, (_req, res, ctx) =>
    res(ctx.status(204))
  ),

  // Simulate a 500 error for error-handling tests
  rest.get(`${THIRD_PARTY_BASE}/error`, (_req, res, ctx) =>
    res(ctx.status(500), ctx.json({ message: 'Internal server error' }))
  ),
];
