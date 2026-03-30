import { createApi } from './index';
import { executeScenario } from './performance';
import {
  createEventFixture,
  createTradeFixture,
  sleep,
} from './test/performanceTestUtils';

describe('API load', () => {
  it('handles concurrent read traffic within a reasonable in-memory latency budget', async () => {
    const api = createApi('http://localhost:3000');

    jest.spyOn(api.trades, 'getTrades').mockImplementation(async () => {
      await sleep(25);
      return [createTradeFixture({ amount: '1' })];
    });

    jest.spyOn(api.events, 'getEvents').mockImplementation(async () => {
      await sleep(25);
      return [createEventFixture()];
    });

    const summary = await executeScenario(
      {
        name: 'concurrent-read-load',
        iterations: 24,
        concurrency: 6,
        thresholds: {
          maxP95LatencyMs: 180,
          maxErrorRate: 0,
          minThroughputPerSecond: 25,
        },
      },
      async (context) => {
        if (context.iteration % 2 === 0) {
          await context.measure('getTrades', () => api.trades.getTrades(1, 0));
        } else {
          await context.measure('getEvents', () => api.events.getEvents({ limit: 1 }));
        }
      }
    );

    expect(summary.sampleCount).toBe(24);
    expect(summary.errorRate).toBe(0);
    expect(summary.p95LatencyMs).toBeLessThan(180);
    expect(summary.throughputPerSecond).toBeGreaterThan(25);
    expect(summary.alerts).toEqual([]);
  });

  it('sustains mixed read and write traffic under a realistic load profile', async () => {
    const api = createApi('http://localhost:3000');

    jest.spyOn(api.trades, 'getTrades').mockImplementation(async () => {
      await sleep(18);
      return [createTradeFixture({ amount: '1' })];
    });

    jest.spyOn(api.trades, 'createTrade').mockImplementation(async (input) => {
      await sleep(26);
      return createTradeFixture({ id: '2', ...input });
    });

    jest.spyOn(api.events, 'getEvents').mockImplementation(async () => {
      await sleep(24);
      return [createEventFixture()];
    });

    jest.spyOn(api.blockchain, 'getTransactionStatus').mockImplementation(async () => {
      await sleep(15);
      return { status: 'confirmed', confirmed: true };
    });

    const summary = await executeScenario(
      {
        name: 'mixed-load-profile',
        iterations: 32,
        concurrency: 8,
        thresholds: {
          maxP95LatencyMs: 180,
          maxErrorRate: 0,
          minThroughputPerSecond: 35,
        },
      },
      async (context) => {
        switch (context.iteration % 4) {
          case 0:
            await context.measure('getTrades', () => api.trades.getTrades(10, context.iteration));
            break;
          case 1:
            await context.measure('createTrade', () =>
              api.trades.createTrade({
                seller: 'seller',
                buyer: 'buyer',
                amount: `${100 + context.iteration}`,
              })
            );
            break;
          case 2:
            await context.measure('getEvents', () => api.events.getEvents({ limit: 10 }));
            break;
          default:
            await context.measure('getTransactionStatus', () =>
              api.blockchain.getTransactionStatus(`tx-${context.iteration}`)
            );
        }
      }
    );

    expect(summary.sampleCount).toBe(32);
    expect(summary.errorRate).toBe(0);
    expect(summary.operations.map((operation) => operation.operation)).toEqual([
      'createTrade',
      'getEvents',
      'getTrades',
      'getTransactionStatus',
    ]);
    expect(summary.alerts).toEqual([]);
  });
});
