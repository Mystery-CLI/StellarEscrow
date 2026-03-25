/**
 * StellarEscrow Performance Module
 * - Web Vitals monitoring (LCP, FID, CLS, TTFB, FCP)
 * - Route-level lazy loading via IntersectionObserver
 * - Resource prefetching on idle
 * - In-memory API response cache with TTL
 */

// ── Web Vitals ────────────────────────────────────────────────────────────────

const metrics = {};

function recordMetric(name, value, rating) {
  metrics[name] = { value, rating, ts: Date.now() };
  if (process.env?.NODE_ENV !== 'production') {
    console.debug(`[perf] ${name}: ${Math.round(value)}ms (${rating})`);
  }
  // Beacon to analytics endpoint when available
  if (navigator.sendBeacon) {
    navigator.sendBeacon('/api/metrics', JSON.stringify({ name, value, rating }));
  }
}

function rate(name, value) {
  const thresholds = { LCP: [2500, 4000], FID: [100, 300], CLS: [0.1, 0.25], TTFB: [800, 1800], FCP: [1800, 3000] };
  const [good, poor] = thresholds[name] ?? [Infinity, Infinity];
  return value <= good ? 'good' : value <= poor ? 'needs-improvement' : 'poor';
}

export function observeWebVitals() {
  // TTFB via Navigation Timing
  const nav = performance.getEntriesByType('navigation')[0];
  if (nav) recordMetric('TTFB', nav.responseStart - nav.requestStart, rate('TTFB', nav.responseStart - nav.requestStart));

  // FCP via PerformanceObserver
  observe('paint', (entries) => {
    const fcp = entries.find((e) => e.name === 'first-contentful-paint');
    if (fcp) recordMetric('FCP', fcp.startTime, rate('FCP', fcp.startTime));
  });

  // LCP
  observe('largest-contentful-paint', (entries) => {
    const last = entries[entries.length - 1];
    recordMetric('LCP', last.startTime, rate('LCP', last.startTime));
  });

  // CLS
  let clsValue = 0;
  observe('layout-shift', (entries) => {
    entries.forEach((e) => { if (!e.hadRecentInput) clsValue += e.value; });
    recordMetric('CLS', clsValue, rate('CLS', clsValue));
  });

  // FID / INP
  observe('first-input', (entries) => {
    const fid = entries[0].processingStart - entries[0].startTime;
    recordMetric('FID', fid, rate('FID', fid));
  });
}

function observe(type, cb) {
  try {
    new PerformanceObserver((list) => cb(list.getEntries())).observe({ type, buffered: true });
  } catch {
    // PerformanceObserver not supported for this entry type
  }
}

export function getMetrics() {
  return { ...metrics };
}

// ── Lazy route loading ────────────────────────────────────────────────────────

const ROUTE_MODULES = {
  disputes:   () => import('./disputes.js'),
  arbitrator: () => import('./arbitrator.js'),
  notifications: () => import('./notifications.js'),
};

const loaded = new Set();

/**
 * Observe route sections and load their JS module when they enter the viewport.
 * Each section must have a data-route attribute matching a key in ROUTE_MODULES.
 */
export function initLazyRoutes() {
  const io = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        const route = entry.target.dataset.route;
        if (route && !loaded.has(route) && ROUTE_MODULES[route]) {
          loaded.add(route);
          ROUTE_MODULES[route]()
            .then(() => console.debug(`[perf] lazy loaded: ${route}`))
            .catch((err) => console.warn(`[perf] failed to load ${route}:`, err));
          io.unobserve(entry.target);
        }
      });
    },
    { rootMargin: '200px' }   // start loading 200px before visible
  );

  document.querySelectorAll('[data-route]').forEach((el) => io.observe(el));
}

// ── Idle prefetch ─────────────────────────────────────────────────────────────

const PREFETCH_ROUTES = ['/api/events?limit=50', '/api/health'];

export function prefetchOnIdle() {
  const schedule = window.requestIdleCallback ?? ((cb) => setTimeout(cb, 1000));
  schedule(() => {
    PREFETCH_ROUTES.forEach((url) => {
      const link = document.createElement('link');
      link.rel = 'prefetch';
      link.href = url;
      link.as = 'fetch';
      link.crossOrigin = 'anonymous';
      document.head.appendChild(link);
    });
  });
}

// ── API response cache (in-memory, TTL-based) ─────────────────────────────────

const _cache = new Map();

/**
 * Fetch with in-memory TTL cache.
 * @param {string} url
 * @param {RequestInit} [options]
 * @param {number} [ttl=30000] - milliseconds
 */
export async function cachedFetch(url, options = {}, ttl = 30_000) {
  const key = url;
  const cached = _cache.get(key);
  if (cached && Date.now() - cached.ts < ttl) return cached.data;

  const res = await fetch(url, options);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const data = await res.json();
  _cache.set(key, { data, ts: Date.now() });
  return data;
}

export function invalidateCache(url) {
  if (url) _cache.delete(url);
  else _cache.clear();
}
