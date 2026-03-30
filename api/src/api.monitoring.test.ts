import { createApi, EscrowApi } from './index';
import {
  evaluateThresholds,
  executeScenario,
  PerformanceMonitor,
} from './performance';
import {
  createEventFixture,
  createTradeFixture,
  sleep,
} from './test/performanceTestUtils';

describe('API performance monitoring', () => {
  it('captures per-operation metrics and produces threshold alerts', async () => {
    const api = createApi('http://localhost:3000');
    const monitor = new PerformanceMonitor();

    jest.spyOn(api.trades, 'getTrades').mockImplementation(async () => {
      await sleep(35);
      return [createTradeFixture({ amount: '10' })];
    });

    jest.spyOn(api.events, 'getEvents').mockImplementation(async () => {
      await sleep(85);
      return [createEventFixture()];
    });

    const summary = await executeScenario(
      {
        name: 'monitored-mix',
        iterations: 20,
        concurrency: 5,
      },
      async (context) => {
        if (context.iteration % 2 === 0) {
          await context.measure('getTrades', () => api.trades.getTrades(10, context.iteration));
        } else {
          await context.measure('getEvents', () => api.events.getEvents({ limit: 10 }));
        }
      },
      monitor
    );

    const alerts = evaluateThresholds(summary, {
      maxP95LatencyMs: 70,
      minThroughputPerSecond: 30,
    });

    const getEventsMetrics = summary.operations.find((operation) => operation.operation === 'getEvents');
    const getTradesMetrics = summary.operations.find((operation) => operation.operation === 'getTrades');

    expect(summary.operations.map((operation) => operation.operation)).toEqual([
      'getEvents',
      'getTrades',
    ]);
    expect(getEventsMetrics?.avgLatencyMs ?? 0).toBeGreaterThan(getTradesMetrics?.avgLatencyMs ?? 0);
    expect(alerts.map((alert) => alert.metric)).toContain('maxP95LatencyMs');
  });

  it('records failed requests for observability without aborting the scenario', async () => {
    const api = new EscrowApi({
      baseURL: 'http://localhost:3000/api',
      timeout: 1_000,
      mockEnabled: false,
      retryConfig: {
        maxRetries: 0,
        delayMs: 1,
        backoffMultiplier: 1,
      },
    });

    jest.spyOn(api.events, 'getEvents').mockImplementation(async () => {
      await sleep(15);
      throw {
        message: 'internal',
        status: 500,
        code: 'INTERNAL_ERROR',
      };
    });

    const summary = await executeScenario(
      {
        name: 'error-observability',
        iterations: 6,
        concurrency: 2,
        thresholds: {
          maxErrorRate: 0.2,
        },
      },
      async (context) => {
        await context.measure('getEvents', () => api.events.getEvents({ limit: 5 }));
      }
    );

    expect(summary.sampleCount).toBe(6);
    expect(summary.errorCount).toBe(6);
    expect(summary.errors[0]?.status).toBe(500);
    expect(summary.alerts.map((alert) => alert.metric)).toContain('maxErrorRate');
  });
});
