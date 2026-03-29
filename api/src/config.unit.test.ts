import { createApi, loadConfig } from './index';

describe('API config unit', () => {
  const originalEnv = process.env;

  beforeEach(() => {
    process.env = { ...originalEnv };
  });

  afterAll(() => {
    process.env = originalEnv;
  });

  it('loads default config values', () => {
    expect(loadConfig()).toMatchObject({
      baseUrl: 'http://localhost:3000/api',
      wsUrl: 'ws://localhost:3000/api/ws',
      timeoutMs: 30000,
      retryMax: 3,
      retryDelayMs: 1000,
      retryBackoffMultiplier: 2,
      mockEnabled: false,
    });
  });

  it('lets environment variables override defaults', () => {
    process.env.STELLAR_ESCROW_API_BASE_URL = 'https://api.example.com/api';
    process.env.STELLAR_ESCROW_TIMEOUT_MS = '45000';
    process.env.STELLAR_ESCROW_RETRY_MAX = '5';
    process.env.STELLAR_ESCROW_MOCK = 'true';

    expect(loadConfig()).toMatchObject({
      baseUrl: 'https://api.example.com/api',
      timeoutMs: 45000,
      retryMax: 5,
      mockEnabled: true,
    });
  });

  it('rejects invalid config overrides', () => {
    expect(() => loadConfig({ timeoutMs: 500 })).toThrow('timeoutMs must be >= 1000');
    expect(() => loadConfig({ retryBackoffMultiplier: 0 })).toThrow(
      'retryBackoffMultiplier must be >= 1'
    );
  });

  it('normalizes bare origins to the /api base path', async () => {
    const api = createApi('http://localhost:3000');

    await expect(api.trades.getTrades(1, 0)).resolves.toHaveLength(1);
  });
});
