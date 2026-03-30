import { rest } from 'msw';
import { server } from './mocks';
import {
  ApiConnector,
  IntegrationConfig,
  IntegrationMonitor,
  IntegrationService,
} from './integration';

// ── Shared fixture ────────────────────────────────────────────────────────────

const BASE_CONFIG: IntegrationConfig = {
  id: 'test-integration',
  name: 'Test Integration',
  provider: 'custom',
  baseUrl: 'https://api.example-integration.com',
  apiKey: 'secret-token',
};

// ── ApiConnector ──────────────────────────────────────────────────────────────

describe('ApiConnector', () => {
  describe('HTTP methods', () => {
    it('GET returns data from the third-party API', async () => {
      const connector = new ApiConnector(BASE_CONFIG);
      const data = await connector.get<{ items: unknown[] }>('/resource');
      expect(data.items).toHaveLength(1);
    });

    it('POST creates a resource', async () => {
      const connector = new ApiConnector(BASE_CONFIG);
      const data = await connector.post<{ id: string; name: string }>('/resource', { name: 'foo' });
      expect(data.id).toBe('r-new');
      expect(data.name).toBe('foo');
    });

    it('PATCH updates a resource', async () => {
      const connector = new ApiConnector(BASE_CONFIG);
      const data = await connector.patch<{ id: string; name: string }>('/resource/r1', { name: 'bar' });
      expect(data.id).toBe('r1');
      expect(data.name).toBe('bar');
    });

    it('DELETE removes a resource without error', async () => {
      const connector = new ApiConnector(BASE_CONFIG);
      await expect(connector.delete('/resource/r1')).resolves.not.toThrow();
    });
  });

  describe('metrics tracking', () => {
    it('starts with zero counts', () => {
      const connector = new ApiConnector(BASE_CONFIG);
      const m = connector.getMetrics();
      expect(m.totalRequests).toBe(0);
      expect(m.successCount).toBe(0);
      expect(m.errorCount).toBe(0);
    });

    it('increments success counts after a successful call', async () => {
      const connector = new ApiConnector(BASE_CONFIG);
      await connector.get('/resource');
      const m = connector.getMetrics();
      expect(m.totalRequests).toBe(1);
      expect(m.successCount).toBe(1);
      expect(m.errorCount).toBe(0);
      expect(m.lastRequestAt).not.toBeNull();
    });

    it('increments error counts when the call fails', async () => {
      const connector = new ApiConnector(BASE_CONFIG);
      await expect(connector.get('/error')).rejects.toBeDefined();
      const m = connector.getMetrics();
      expect(m.totalRequests).toBe(1);
      expect(m.errorCount).toBe(1);
      expect(m.successCount).toBe(0);
    });

    it('accumulates latency across multiple calls', async () => {
      const connector = new ApiConnector(BASE_CONFIG);
      await connector.get('/resource');
      await connector.get('/resource');
      const m = connector.getMetrics();
      expect(m.totalRequests).toBe(2);
      expect(m.successCount).toBe(2);
      expect(m.totalLatencyMs).toBeGreaterThanOrEqual(0);
    });
  });

  describe('auth headers', () => {
    it('attaches Authorization header when apiKey is set', async () => {
      let capturedAuthHeader: string | undefined;

      server.use(
        rest.get('https://api.example-integration.com/resource', (req, res, ctx) => {
          capturedAuthHeader = req.headers.get('Authorization') ?? undefined;
          return res(ctx.json({ items: [] }));
        })
      );

      const connector = new ApiConnector(BASE_CONFIG);
      await connector.get('/resource');

      expect(capturedAuthHeader).toBe('Bearer secret-token');
    });

    it('merges extra static headers from config', async () => {
      let capturedHeader: string | undefined;

      server.use(
        rest.get('https://api.example-integration.com/resource', (req, res, ctx) => {
          capturedHeader = req.headers.get('X-Custom') ?? undefined;
          return res(ctx.json({ items: [] }));
        })
      );

      const connector = new ApiConnector({
        ...BASE_CONFIG,
        headers: { 'X-Custom': 'my-value' },
      });
      await connector.get('/resource');

      expect(capturedHeader).toBe('my-value');
    });
  });
});

// ── IntegrationMonitor ────────────────────────────────────────────────────────

describe('IntegrationMonitor', () => {
  it('records a healthy status after a successful health check', async () => {
    const monitor = new IntegrationMonitor();
    const connector = new ApiConnector(BASE_CONFIG);

    monitor.start(connector, '/health', 999_999);
    await new Promise((r) => setTimeout(r, 100));

    const health = monitor.getHealth(BASE_CONFIG.id);
    expect(health).toBeDefined();
    expect(health?.status).toBe('active');
    expect(health?.latencyMs).toBeGreaterThanOrEqual(0);
    monitor.stopAll();
  });

  it('records an error status when the health endpoint fails', async () => {
    server.use(
      rest.get('https://api.example-integration.com/health', (_req, res, ctx) =>
        res(ctx.status(503), ctx.json({ message: 'Service unavailable' }))
      )
    );

    const monitor = new IntegrationMonitor();
    const connector = new ApiConnector(BASE_CONFIG);
    monitor.start(connector, '/health', 999_999);
    await new Promise((r) => setTimeout(r, 100));

    const health = monitor.getHealth(BASE_CONFIG.id);
    expect(health?.status).toBe('error');
    expect(health?.error).toBeDefined();
    monitor.stopAll();
  });

  it('getAllHealth returns records for all monitored integrations', async () => {
    const monitor = new IntegrationMonitor();
    const c1 = new ApiConnector({ ...BASE_CONFIG, id: 'int-a' });
    const c2 = new ApiConnector({ ...BASE_CONFIG, id: 'int-b' });
    monitor.start(c1, '/health', 999_999);
    monitor.start(c2, '/health', 999_999);
    await new Promise((r) => setTimeout(r, 100));
    expect(monitor.getAllHealth()).toHaveLength(2);
    monitor.stopAll();
  });
});

// ── IntegrationService ────────────────────────────────────────────────────────

describe('IntegrationService', () => {
  let service: IntegrationService;

  beforeEach(() => {
    service = new IntegrationService();
  });

  afterEach(() => {
    service.clear();
  });

  describe('register', () => {
    it('registers a connector and returns it', () => {
      const connector = service.register(BASE_CONFIG);
      expect(connector).toBeInstanceOf(ApiConnector);
    });

    it('throws when registering a duplicate id', () => {
      service.register(BASE_CONFIG);
      expect(() => service.register(BASE_CONFIG)).toThrow(/already registered/);
    });

    it('throws when the integration is disabled', () => {
      expect(() =>
        service.register({ ...BASE_CONFIG, id: 'disabled', enabled: false })
      ).toThrow(/disabled/);
    });
  });

  describe('connector', () => {
    it('returns the registered connector', () => {
      service.register(BASE_CONFIG);
      expect(service.connector(BASE_CONFIG.id)).toBeInstanceOf(ApiConnector);
    });

    it('throws when the id is not registered', () => {
      expect(() => service.connector('unknown')).toThrow(/not registered/);
    });
  });

  describe('has', () => {
    it('returns true for a registered integration', () => {
      service.register(BASE_CONFIG);
      expect(service.has(BASE_CONFIG.id)).toBe(true);
    });

    it('returns false for an unregistered id', () => {
      expect(service.has('not-here')).toBe(false);
    });
  });

  describe('deregister', () => {
    it('removes the integration', () => {
      service.register(BASE_CONFIG);
      service.deregister(BASE_CONFIG.id);
      expect(service.has(BASE_CONFIG.id)).toBe(false);
    });

    it('allows re-registration after deregister', () => {
      service.register(BASE_CONFIG);
      service.deregister(BASE_CONFIG.id);
      expect(() => service.register(BASE_CONFIG)).not.toThrow();
    });
  });

  describe('getMetrics', () => {
    it('returns empty array when no integrations are registered', () => {
      expect(service.getMetrics()).toEqual([]);
    });

    it('returns metrics for all registered integrations', () => {
      service.register(BASE_CONFIG);
      service.register({ ...BASE_CONFIG, id: 'second' });
      const metrics = service.getMetrics();
      expect(metrics).toHaveLength(2);
      expect(metrics.map((m) => m.integrationId)).toContain(BASE_CONFIG.id);
    });
  });

  describe('getEventLog', () => {
    it('logs a REGISTER event when an integration is registered', () => {
      service.register(BASE_CONFIG);
      const log = service.getEventLog();
      expect(log).toHaveLength(1);
      expect(log[0].method).toBe('REGISTER');
      expect(log[0].integrationId).toBe(BASE_CONFIG.id);
    });
  });

  describe('end-to-end', () => {
    it('calls the third-party API through the service', async () => {
      service.register(BASE_CONFIG);
      const connector = service.connector(BASE_CONFIG.id);
      const data = await connector.get<{ items: unknown[] }>('/resource');
      expect(data.items).toHaveLength(1);

      const metrics = service.getMetrics().find((m) => m.integrationId === BASE_CONFIG.id);
      expect(metrics?.successCount).toBe(1);
    });
  });
});
