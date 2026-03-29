export interface PerformanceThresholds {
  maxAvgLatencyMs?: number;
  maxP95LatencyMs?: number;
  maxMaxLatencyMs?: number;
  maxErrorRate?: number;
  minThroughputPerSecond?: number;
}

export interface PerformanceAlert {
  severity: 'warning' | 'critical';
  metric: keyof PerformanceThresholds;
  message: string;
  observed: number;
  threshold: number;
}

export interface PerformanceError {
  message: string;
  code?: string;
  status?: number;
}

export interface PerformanceSample {
  scenario: string;
  operation: string;
  iteration: number;
  startedAt: number;
  endedAt: number;
  durationMs: number;
  success: boolean;
  warmup: boolean;
  metadata?: Record<string, unknown>;
  error?: PerformanceError;
}

export interface OperationPerformanceSummary {
  operation: string;
  sampleCount: number;
  successCount: number;
  errorCount: number;
  errorRate: number;
  avgLatencyMs: number;
  p95LatencyMs: number;
  maxLatencyMs: number;
}

export interface PerformanceSummary {
  scenario: string;
  concurrency: number;
  iterations: number;
  warmupIterations: number;
  sampleCount: number;
  successCount: number;
  errorCount: number;
  errorRate: number;
  totalDurationMs: number;
  wallTimeMs: number;
  minLatencyMs: number;
  avgLatencyMs: number;
  p50LatencyMs: number;
  p95LatencyMs: number;
  maxLatencyMs: number;
  throughputPerSecond: number;
  operations: OperationPerformanceSummary[];
  errors: PerformanceError[];
  alerts: PerformanceAlert[];
}

export interface ScenarioExecutionContext {
  scenario: string;
  iteration: number;
  warmup: boolean;
  monitor: PerformanceMonitor;
  measure<T>(
    operation: string,
    task: () => Promise<T>,
    metadata?: Record<string, unknown>
  ): Promise<T>;
}

export interface PerformanceScenarioConfig {
  name: string;
  iterations: number;
  concurrency?: number;
  warmupIterations?: number;
  thresholds?: PerformanceThresholds;
  failFast?: boolean;
}

function percentile(values: number[], ratio: number): number {
  if (values.length === 0) {
    return 0;
  }

  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(sorted.length - 1, Math.ceil(sorted.length * ratio) - 1);
  return sorted[index];
}

function average(values: number[]): number {
  if (values.length === 0) {
    return 0;
  }

  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function normalizeError(error: unknown): PerformanceError {
  if (typeof error === 'object' && error !== null) {
    const candidate = error as { message?: string; code?: string; status?: number };

    return {
      message: candidate.message ?? 'Unknown error',
      code: candidate.code,
      status: candidate.status,
    };
  }

  return { message: String(error) };
}

function buildOperationSummary(samples: PerformanceSample[]): OperationPerformanceSummary[] {
  const grouped = new Map<string, PerformanceSample[]>();

  for (const sample of samples) {
    const existing = grouped.get(sample.operation) ?? [];
    existing.push(sample);
    grouped.set(sample.operation, existing);
  }

  return [...grouped.entries()]
    .map(([operation, operationSamples]) => {
      const durations = operationSamples.map((sample) => sample.durationMs);
      const errorCount = operationSamples.filter((sample) => !sample.success).length;

      return {
        operation,
        sampleCount: operationSamples.length,
        successCount: operationSamples.length - errorCount,
        errorCount,
        errorRate: operationSamples.length === 0 ? 0 : errorCount / operationSamples.length,
        avgLatencyMs: average(durations),
        p95LatencyMs: percentile(durations, 0.95),
        maxLatencyMs: durations.length === 0 ? 0 : Math.max(...durations),
      };
    })
    .sort((left, right) => left.operation.localeCompare(right.operation));
}

export function evaluateThresholds(
  summary: PerformanceSummary,
  thresholds: PerformanceThresholds = {}
): PerformanceAlert[] {
  const alerts: PerformanceAlert[] = [];

  if (
    thresholds.maxAvgLatencyMs !== undefined &&
    summary.avgLatencyMs > thresholds.maxAvgLatencyMs
  ) {
    alerts.push({
      severity: 'warning',
      metric: 'maxAvgLatencyMs',
      message: `Average latency ${summary.avgLatencyMs.toFixed(2)}ms exceeds ${thresholds.maxAvgLatencyMs}ms`,
      observed: summary.avgLatencyMs,
      threshold: thresholds.maxAvgLatencyMs,
    });
  }

  if (
    thresholds.maxP95LatencyMs !== undefined &&
    summary.p95LatencyMs > thresholds.maxP95LatencyMs
  ) {
    alerts.push({
      severity: 'critical',
      metric: 'maxP95LatencyMs',
      message: `P95 latency ${summary.p95LatencyMs.toFixed(2)}ms exceeds ${thresholds.maxP95LatencyMs}ms`,
      observed: summary.p95LatencyMs,
      threshold: thresholds.maxP95LatencyMs,
    });
  }

  if (
    thresholds.maxMaxLatencyMs !== undefined &&
    summary.maxLatencyMs > thresholds.maxMaxLatencyMs
  ) {
    alerts.push({
      severity: 'warning',
      metric: 'maxMaxLatencyMs',
      message: `Max latency ${summary.maxLatencyMs.toFixed(2)}ms exceeds ${thresholds.maxMaxLatencyMs}ms`,
      observed: summary.maxLatencyMs,
      threshold: thresholds.maxMaxLatencyMs,
    });
  }

  if (thresholds.maxErrorRate !== undefined && summary.errorRate > thresholds.maxErrorRate) {
    alerts.push({
      severity: 'critical',
      metric: 'maxErrorRate',
      message: `Error rate ${(summary.errorRate * 100).toFixed(2)}% exceeds ${(thresholds.maxErrorRate * 100).toFixed(2)}%`,
      observed: summary.errorRate,
      threshold: thresholds.maxErrorRate,
    });
  }

  if (
    thresholds.minThroughputPerSecond !== undefined &&
    summary.throughputPerSecond < thresholds.minThroughputPerSecond
  ) {
    alerts.push({
      severity: 'warning',
      metric: 'minThroughputPerSecond',
      message: `Throughput ${summary.throughputPerSecond.toFixed(2)} req/s is below ${thresholds.minThroughputPerSecond} req/s`,
      observed: summary.throughputPerSecond,
      threshold: thresholds.minThroughputPerSecond,
    });
  }

  return alerts;
}

export class PerformanceMonitor {
  private samples: PerformanceSample[] = [];

  async measure<T>(
    sample: Omit<PerformanceSample, 'startedAt' | 'endedAt' | 'durationMs' | 'success' | 'error'>,
    task: () => Promise<T>
  ): Promise<T> {
    const startedAt = performance.now();

    try {
      const result = await task();
      const endedAt = performance.now();

      this.samples.push({
        ...sample,
        startedAt,
        endedAt,
        durationMs: endedAt - startedAt,
        success: true,
      });

      return result;
    } catch (error) {
      const endedAt = performance.now();

      this.samples.push({
        ...sample,
        startedAt,
        endedAt,
        durationMs: endedAt - startedAt,
        success: false,
        error: normalizeError(error),
      });

      throw error;
    }
  }

  getSamples(filter?: {
    scenario?: string;
    operation?: string;
    includeWarmup?: boolean;
  }): PerformanceSample[] {
    return this.samples.filter((sample) => {
      if (filter?.scenario && sample.scenario !== filter.scenario) {
        return false;
      }

      if (filter?.operation && sample.operation !== filter.operation) {
        return false;
      }

      if (!filter?.includeWarmup && sample.warmup) {
        return false;
      }

      return true;
    });
  }

  summarize(input: {
    scenario: string;
    concurrency: number;
    iterations: number;
    warmupIterations?: number;
    wallTimeMs: number;
  }): PerformanceSummary {
    const samples = this.getSamples({ scenario: input.scenario });
    const durations = samples.map((sample) => sample.durationMs);
    const errorSamples = samples.filter((sample) => !sample.success);

    return {
      scenario: input.scenario,
      concurrency: input.concurrency,
      iterations: input.iterations,
      warmupIterations: input.warmupIterations ?? 0,
      sampleCount: samples.length,
      successCount: samples.length - errorSamples.length,
      errorCount: errorSamples.length,
      errorRate: samples.length === 0 ? 0 : errorSamples.length / samples.length,
      totalDurationMs: durations.reduce((sum, duration) => sum + duration, 0),
      wallTimeMs: input.wallTimeMs,
      minLatencyMs: durations.length === 0 ? 0 : Math.min(...durations),
      avgLatencyMs: average(durations),
      p50LatencyMs: percentile(durations, 0.5),
      p95LatencyMs: percentile(durations, 0.95),
      maxLatencyMs: durations.length === 0 ? 0 : Math.max(...durations),
      throughputPerSecond:
        input.wallTimeMs <= 0 ? 0 : samples.length / (input.wallTimeMs / 1000),
      operations: buildOperationSummary(samples),
      errors: errorSamples.map((sample) => sample.error ?? { message: 'Unknown error' }),
      alerts: [],
    };
  }
}

export async function executeScenario(
  config: PerformanceScenarioConfig,
  task: (context: ScenarioExecutionContext) => Promise<void>,
  monitor = new PerformanceMonitor()
): Promise<PerformanceSummary> {
  const warmupIterations = config.warmupIterations ?? 0;
  const totalIterations = config.iterations + warmupIterations;
  const concurrency = Math.max(1, Math.min(config.concurrency ?? 1, totalIterations || 1));
  let nextIteration = 0;
  let failure: unknown;

  const wallStartedAt = performance.now();

  const worker = async () => {
    while (true) {
      const rawIteration = nextIteration;
      nextIteration += 1;

      if (rawIteration >= totalIterations) {
        return;
      }

      const warmup = rawIteration < warmupIterations;
      const iteration = warmup ? rawIteration : rawIteration - warmupIterations;

      const context: ScenarioExecutionContext = {
        scenario: config.name,
        iteration,
        warmup,
        monitor,
        measure: <T>(
          operation: string,
          measuredTask: () => Promise<T>,
          metadata?: Record<string, unknown>
        ) =>
          monitor.measure(
            {
              scenario: config.name,
              operation,
              iteration,
              warmup,
              metadata,
            },
            measuredTask
          ),
      };

      try {
        await task(context);
      } catch (error) {
        if (config.failFast) {
          failure = error;
          throw error;
        }
      }
    }
  };

  try {
    await Promise.all(Array.from({ length: concurrency }, () => worker()));
  } catch (error) {
    failure = error;
  }

  const wallTimeMs = performance.now() - wallStartedAt;
  const summary = monitor.summarize({
    scenario: config.name,
    concurrency,
    iterations: config.iterations,
    warmupIterations,
    wallTimeMs,
  });

  summary.alerts = evaluateThresholds(summary, config.thresholds);

  if (failure && config.failFast) {
    throw failure;
  }

  return summary;
}
