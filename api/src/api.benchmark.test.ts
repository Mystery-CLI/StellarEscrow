import { createApi } from './index';
import { executeScenario } from './performance';
import { createTradeFixture, sleep } from './test/performanceTestUtils';

describe('API benchmark', () => {
  it('benchmarks trade listing latency against a stable baseline', async () => {
    const api = createApi('http://localhost:3000');

    jest.spyOn(api.trades, 'getTrades').mockImplementation(async () => {
      await sleep(18);
      return [createTradeFixture({ amount: '10' })];
    });

    const summary = await executeScenario(
      {
        name: 'trade-list-baseline',
        iterations: 18,
        warmupIterations: 3,
        thresholds: {
          maxAvgLatencyMs: 120,
          maxP95LatencyMs: 175,
        },
      },
      async (context) => {
        await context.measure('getTrades', () => api.trades.getTrades(20, context.iteration));
      }
    );

    expect(summary.sampleCount).toBe(18);
    expect(summary.avgLatencyMs).toBeLessThan(120);
    expect(summary.p95LatencyMs).toBeLessThan(175);
    expect(summary.alerts).toEqual([]);
  });

  it('benchmarks a create and settlement workflow under representative latency', async () => {
    const api = createApi('http://localhost:3000');

    jest.spyOn(api.trades, 'createTrade').mockImplementation(async (input) => {
      await sleep(28);
      return createTradeFixture({ id: '2', ...input });
    });

    jest.spyOn(api.blockchain, 'fundTrade').mockImplementation(async () => {
      await sleep(22);
      return { txHash: '0xfund' };
    });

    jest.spyOn(api.blockchain, 'completeTrade').mockImplementation(async () => {
      await sleep(24);
      return { txHash: '0xcomplete' };
    });

    const summary = await executeScenario(
      {
        name: 'create-settle-workflow',
        iterations: 12,
        thresholds: {
          maxAvgLatencyMs: 110,
          maxP95LatencyMs: 150,
        },
      },
      async (context) => {
        await context.measure('createTrade', () =>
          api.trades.createTrade({
            seller: 'seller',
            buyer: 'buyer',
            amount: `${100 + context.iteration}`,
          })
        );
        await context.measure('fundTrade', () =>
          api.blockchain.fundTrade(`${context.iteration}`, '100')
        );
        await context.measure('completeTrade', () =>
          api.blockchain.completeTrade(`${context.iteration}`)
        );
      }
    );

    expect(summary.sampleCount).toBe(36);
    expect(summary.avgLatencyMs).toBeLessThan(110);
    expect(summary.p95LatencyMs).toBeLessThan(150);
    expect(summary.operations.map((operation) => operation.operation)).toEqual([
      'completeTrade',
      'createTrade',
      'fundTrade',
    ]);
  });
});
