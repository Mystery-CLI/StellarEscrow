/**
 * API Contract Regression Suite
 *
 * Verifies that all public API endpoints return the expected shape,
 * status codes, and required fields. Any breaking change to the API
 * contract will be caught here before it reaches production.
 */

import { api } from '../setup/api-client';

describe('API Contract Regression — Health & Status', () => {
  it('GET /health returns 200 with status field', async () => {
    const res = await api.get('/health');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('status');
  });

  it('GET /status returns indexer sync state', async () => {
    const res = await api.get('/status');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('syncing');
    expect(res.data).toHaveProperty('total_events');
    expect(typeof res.data.total_events).toBe('number');
  });

  it('GET /stats returns event type counts', async () => {
    const res = await api.get('/stats');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('total_events');
    expect(res.data).toHaveProperty('by_type');
    expect(Array.isArray(res.data.by_type)).toBe(true);
  });
});

describe('API Contract Regression — Events', () => {
  it('GET /events returns paginated response', async () => {
    const res = await api.get('/events?limit=5&offset=0');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('data');
    expect(res.data).toHaveProperty('total');
    expect(res.data).toHaveProperty('limit');
    expect(res.data).toHaveProperty('offset');
    expect(Array.isArray(res.data.data)).toBe(true);
  });

  it('GET /events respects limit parameter', async () => {
    const res = await api.get('/events?limit=3');
    expect(res.status).toBe(200);
    expect(res.data.data.length).toBeLessThanOrEqual(3);
    expect(res.data.limit).toBe(3);
  });

  it('GET /events/:id returns 404 for unknown id', async () => {
    const res = await api.get('/events/00000000-0000-0000-0000-000000000000').catch(e => e.response);
    expect(res.status).toBe(404);
  });

  it('GET /events/trade/:trade_id returns array', async () => {
    const res = await api.get('/events/trade/1');
    expect(res.status).toBe(200);
    expect(Array.isArray(res.data.items ?? res.data)).toBe(true);
  });

  it('GET /events/type/:event_type filters correctly', async () => {
    const res = await api.get('/events/type/trade_created');
    expect(res.status).toBe(200);
    const items: any[] = res.data.items ?? res.data.data ?? res.data;
    items.forEach((e: any) => {
      expect(e.event_type).toBe('trade_created');
    });
  });
});

describe('API Contract Regression — Search', () => {
  it('GET /search returns trades, users, arbitrators, suggestions', async () => {
    const res = await api.get('/search?q=test');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('trades');
    expect(res.data).toHaveProperty('users');
    expect(res.data).toHaveProperty('arbitrators');
    expect(res.data).toHaveProperty('suggestions');
  });

  it('GET /search/trades returns array', async () => {
    const res = await api.get('/search/trades?q=');
    expect(res.status).toBe(200);
    expect(Array.isArray(res.data)).toBe(true);
  });
});

describe('API Contract Regression — Compliance', () => {
  it('GET /compliance/status/:address returns status object', async () => {
    const res = await api.get('/compliance/status/GTEST00000000000000000000000000000000000000000000000000');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('status');
  });

  it('GET /compliance/report returns report fields', async () => {
    const res = await api.get('/compliance/report');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('total_checks');
    expect(res.data).toHaveProperty('approved');
    expect(res.data).toHaveProperty('blocked');
    expect(res.data).toHaveProperty('avg_risk_score');
  });
});

describe('API Contract Regression — Monitoring', () => {
  it('GET /monitoring/dashboard returns health_status', async () => {
    const res = await api.get('/monitoring/dashboard');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('health_status');
    expect(res.data).toHaveProperty('active_alert_count');
    expect(res.data).toHaveProperty('metrics');
  });

  it('GET /monitoring/alerts returns array', async () => {
    const res = await api.get('/monitoring/alerts');
    expect(res.status).toBe(200);
    expect(Array.isArray(res.data)).toBe(true);
  });

  it('GET /metrics returns Prometheus text format', async () => {
    const res = await api.get('/metrics');
    expect(res.status).toBe(200);
    expect(res.headers['content-type']).toMatch(/text\/plain/);
  });
});

describe('API Contract Regression — Audit', () => {
  it('GET /audit returns paginated audit logs', async () => {
    const res = await api.get('/audit?limit=5');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('items');
    expect(res.data).toHaveProperty('total');
    expect(Array.isArray(res.data.items)).toBe(true);
  });

  it('GET /audit/stats returns category/outcome breakdowns', async () => {
    const res = await api.get('/audit/stats');
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('total');
    expect(res.data).toHaveProperty('by_category');
    expect(res.data).toHaveProperty('by_outcome');
  });
});
