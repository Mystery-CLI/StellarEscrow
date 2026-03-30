import { browser } from '$app/environment';

export interface Metric { name: string; value: number; rating: 'good' | 'needs-improvement' | 'poor' }

const thresholds: Record<string, [number, number]> = {
  LCP: [2500, 4000],
  FID: [100, 300],
  CLS: [0.1, 0.25],
  FCP: [1800, 3000],
  TTFB: [800, 1800],
};

function rate(name: string, value: number): Metric['rating'] {
  const [good, poor] = thresholds[name] ?? [Infinity, Infinity];
  return value <= good ? 'good' : value <= poor ? 'needs-improvement' : 'poor';
}

/** Collect Web Vitals via PerformanceObserver and report via callback */
export function collectWebVitals(onMetric: (m: Metric) => void) {
  if (!browser || !('PerformanceObserver' in window)) return;

  observe('largest-contentful-paint', (entries) => {
    const e = entries.at(-1) as PerformanceEntry & { renderTime?: number; loadTime?: number };
    const value = (e.renderTime || e.loadTime) ?? 0;
    onMetric({ name: 'LCP', value, rating: rate('LCP', value) });
  });

  observe('first-input', (entries) => {
    const e = entries[0] as PerformanceEntry & { processingStart: number };
    const value = e.processingStart - e.startTime;
    onMetric({ name: 'FID', value, rating: rate('FID', value) });
  });

  let clsValue = 0;
  observe('layout-shift', (entries) => {
    for (const e of entries as (PerformanceEntry & { hadRecentInput: boolean; value: number })[]) {
      if (!e.hadRecentInput) clsValue += e.value;
    }
    onMetric({ name: 'CLS', value: clsValue, rating: rate('CLS', clsValue) });
  });

  observe('paint', (entries) => {
    const fcp = entries.find((e) => e.name === 'first-contentful-paint');
    if (fcp) onMetric({ name: 'FCP', value: fcp.startTime, rating: rate('FCP', fcp.startTime) });
  });

  observe('navigation', (entries) => {
    const e = entries[0] as PerformanceNavigationTiming;
    const value = e.responseStart - e.requestStart;
    onMetric({ name: 'TTFB', value, rating: rate('TTFB', value) });
  });
}

function observe(type: string, cb: (entries: PerformanceEntry[]) => void) {
  try {
    new PerformanceObserver((list) => cb(list.getEntries())).observe({ type, buffered: true });
  } catch { /* unsupported entry type */ }
}

/** Measure an async operation and log duration in dev */
export async function measure<T>(label: string, fn: () => Promise<T>): Promise<T> {
  const start = performance.now();
  const result = await fn();
  if (import.meta.env.DEV) console.debug(`[perf] ${label}: ${(performance.now() - start).toFixed(1)}ms`);
  return result;
}
