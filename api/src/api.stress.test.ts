import { EscrowApi } from './index';
import { executeScenario } from './performance';
import {
  createEventFixture,
  createTradeFixture,
  sleep,
} from './test/performanceTestUtils';

function createStressApi(): EscrowApi {
  return new EscrowApi({
    baseURL: 'http://localhost:3000/api',
    timeout: 1_500,
    mockEnabled: false,
    retryConfig: {
      maxRetries: 2,
      delayMs: 5,
      backoffMultiplier: 1,
    },
  });
}

describe('API stress', () => {
  it('absorbs burst traffic without breaching latency or error budgets', async () => {
    const api = createStressApi();

    jest.spyOn(api.trades, 'getTrades').mockImplementation(async () => {
      await sleep(45);
      return [createTradeFixture({ amount: '25' })];
    });

    jest.spyOn(api.events, 'getEvents').mockImplementation(async () => {
      await sleep(55);
      return [createEventFixture()];
    });

    jest.spyOn(api.blockchain, 'getTransactionStatus').mockImplementation(async () => {
      await sleep(35);
      return { status: 'confirmed', confirmed: true };
    });

    const summary = await executeScenario(
      {
        name: 'burst-traffic',
        iterations: 72,
        concurrency: 12,
        thresholds: {
          maxP95LatencyMs: 275,
          maxErrorRate: 0,
          minThroughputPerSecond: 35,
        },
      },
      async (context) => {
        switch (context.iteration % 3) {
          case 0:
            await context.measure('getTrades', () => api.trades.getTrades(10, 0));
            break;
          case 1:
            await context.measure('getEvents', () => api.events.getEvents(10));
            break;
          default:
            await context.measure('getTransactionStatus', () =>
              api.blockchain.getTransactionStatus(`tx-${context.iteration}`)
            );
        }
      }
    );

    expect(summary.sampleCount).toBe(72);
    expect(summary.errorRate).toBe(0);
    expect(summary.p95LatencyMs).toBeLessThan(275);
    expect(summary.alerts).toEqual([]);
  });

  it('surfaces transient upstream throttling while staying within the stress error budget', async () => {
    const api = createStressApi();
    let attempts = 0;

    jest.spyOn(api.trades, 'getTrades').mockImplementation(async () => {
      attempts += 1;
      await sleep(attempts % 7 === 0 ? 20 : 30);

      if (attempts % 7 === 0) {
        throw {
          message: 'busy',
          status: 503,
          code: 'SERVICE_BUSY',
        };
      }

      return [createTradeFixture({ status: 'funded', amount: '50' })];
    });

    const summary = await executeScenario(
      {
        name: 'retry-under-pressure',
        iterations: 40,
        concurrency: 10,
        thresholds: {
          maxP95LatencyMs: 250,
          maxErrorRate: 0.15,
        },
      },
      async (context) => {
        await context.measure('getTrades', () => api.trades.getTrades(5, context.iteration));
      }
    );

    expect(summary.sampleCount).toBe(40);
    expect(summary.errorCount).toBeGreaterThan(0);
    expect(summary.errorRate).toBeLessThanOrEqual(0.15);
    expect(summary.p95LatencyMs).toBeLessThan(250);
    expect(summary.alerts).toEqual([]);
  });
});
