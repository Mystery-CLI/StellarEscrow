import { createApi } from './index';
import { executeScenario, PerformanceSummary } from './performance';
import { createTradeFixture, sleep } from './test/performanceTestUtils';

async function runScalabilityStep(concurrency: number): Promise<PerformanceSummary> {
  const api = createApi('http://localhost:3000');

  jest.spyOn(api.trades, 'getTrades').mockImplementation(async () => {
    await sleep(28);
    return [createTradeFixture({ amount: '10' })];
  });

  return executeScenario(
    {
      name: `scalability-c${concurrency}`,
      iterations: 32,
      concurrency,
      thresholds: {
        maxP95LatencyMs: 180,
        maxErrorRate: 0,
      },
    },
    async (context) => {
      await context.measure('getTrades', () => api.trades.getTrades(10, context.iteration));
    }
  );
}

describe('API scalability', () => {
  it('increases throughput as concurrency rises without super-linear latency regression', async () => {
    const baseline = await runScalabilityStep(1);
    const medium = await runScalabilityStep(4);
    const high = await runScalabilityStep(8);
    const peak = await runScalabilityStep(16);

    expect(baseline.errorRate).toBe(0);
    expect(peak.errorRate).toBe(0);
    expect(medium.throughputPerSecond).toBeGreaterThan(baseline.throughputPerSecond);
    expect(high.throughputPerSecond).toBeGreaterThan(medium.throughputPerSecond);
    expect(peak.throughputPerSecond).toBeGreaterThan(high.throughputPerSecond);
    expect(peak.avgLatencyMs).toBeLessThan(baseline.avgLatencyMs * 2);
    expect(peak.alerts).toEqual([]);
  });
});
