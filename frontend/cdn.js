/**
 * StellarEscrow CDN Module
 * Handles CDN asset resolution, cache invalidation, monitoring, and geo-distribution.
 */

// ── CDN Configuration ─────────────────────────────────────────────────────────

const CDN_CONFIG = {
  // Primary CDN base URL (override via env or meta tag)
  baseUrl: document.querySelector('meta[name="cdn-url"]')?.content ?? '',

  // Geographic edge regions (ordered by priority)
  regions: ['us-east', 'eu-west', 'ap-southeast', 'sa-east'],

  // Asset cache TTLs (seconds)
  ttl: {
    immutable: 31536000, // 1 year — hashed assets (app.abc123.js)
    static:    86400,    // 1 day  — versioned assets (styles.css)
    dynamic:   300,      // 5 min  — API responses
  },
};

// ── Asset URL Resolution ──────────────────────────────────────────────────────

/**
 * Resolve an asset path to its CDN URL.
 * Falls back to the local path if no CDN base is configured.
 * @param {string} path - e.g. '/app.js', '/styles.css'
 * @returns {string}
 */
export function cdnUrl(path) {
  if (!CDN_CONFIG.baseUrl) return path;
  return `${CDN_CONFIG.baseUrl}${path}`;
}

/**
 * Rewrite all <img>, <link>, and <script> src/href attributes to CDN URLs.
 * Call once after DOM is ready.
 */
export function rewriteAssetUrls() {
  if (!CDN_CONFIG.baseUrl) return;

  document.querySelectorAll('img[src], link[href], script[src]').forEach((el) => {
    const attr = el.tagName === 'LINK' ? 'href' : 'src';
    const val = el.getAttribute(attr);
    if (val && val.startsWith('/') && !val.startsWith('//')) {
      el.setAttribute(attr, cdnUrl(val));
    }
  });
}

// ── Cache Invalidation ────────────────────────────────────────────────────────

/**
 * Invalidate CDN-cached assets by appending a bust token.
 * Useful after deployments when content-hashed filenames aren't used.
 * @param {string[]} paths - asset paths to bust
 * @param {string} [token] - cache-bust token (defaults to current timestamp)
 */
export async function invalidateCdnCache(paths, token = Date.now().toString(36)) {
  if (!CDN_CONFIG.baseUrl) return;

  const results = await Promise.allSettled(
    paths.map((path) =>
      fetch(`${CDN_CONFIG.baseUrl}${path}?_cb=${token}`, {
        method: 'HEAD',
        cache: 'no-store',
      })
    )
  );

  const failed = paths.filter((_, i) => results[i].status === 'rejected');
  if (failed.length) {
    console.warn('[cdn] cache invalidation failed for:', failed);
  }

  return { token, invalidated: paths.length - failed.length, failed };
}

/**
 * Notify the API to purge CDN cache for given paths (server-side purge).
 * @param {string[]} paths
 */
export async function purgeCdnPaths(paths) {
  const res = await fetch('/api/cdn/purge', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ paths }),
  });
  if (!res.ok) throw new Error(`CDN purge failed: HTTP ${res.status}`);
  return res.json();
}

// ── Geographic Distribution ───────────────────────────────────────────────────

/**
 * Detect the nearest CDN edge region using the Timing API.
 * Pings each region's health endpoint and picks the fastest.
 * @returns {Promise<string>} region name
 */
export async function detectNearestRegion() {
  if (!CDN_CONFIG.baseUrl) return 'default';

  const results = await Promise.allSettled(
    CDN_CONFIG.regions.map(async (region) => {
      const start = performance.now();
      await fetch(`${CDN_CONFIG.baseUrl}/health?region=${region}`, {
        method: 'HEAD',
        cache: 'no-store',
        signal: AbortSignal.timeout(3000),
      });
      return { region, latency: performance.now() - start };
    })
  );

  const successful = results
    .filter((r) => r.status === 'fulfilled')
    .map((r) => r.value)
    .sort((a, b) => a.latency - b.latency);

  const nearest = successful[0]?.region ?? CDN_CONFIG.regions[0];
  recordCdnMetric('nearest_region', nearest, successful[0]?.latency ?? 0);
  return nearest;
}

// ── CDN Monitoring ────────────────────────────────────────────────────────────

const _cdnMetrics = [];

function recordCdnMetric(type, value, latency = 0) {
  const entry = { type, value, latency, ts: Date.now() };
  _cdnMetrics.push(entry);

  if (navigator.sendBeacon) {
    navigator.sendBeacon('/api/metrics', JSON.stringify({ source: 'cdn', ...entry }));
  }
}

/**
 * Wrap fetch() to measure CDN hit/miss and latency for asset requests.
 * @param {string} url
 * @param {RequestInit} [options]
 */
export async function monitoredFetch(url, options = {}) {
  const start = performance.now();
  const res = await fetch(url, options);
  const latency = performance.now() - start;

  const cacheStatus = res.headers.get('x-cache') ?? res.headers.get('cf-cache-status') ?? 'unknown';
  const hit = /hit/i.test(cacheStatus);

  recordCdnMetric(hit ? 'cache_hit' : 'cache_miss', url, latency);
  return res;
}

/**
 * Observe resource timing entries for CDN-served assets and report metrics.
 * Call once after page load.
 */
export function observeCdnPerformance() {
  if (!CDN_CONFIG.baseUrl) return;

  try {
    new PerformanceObserver((list) => {
      list.getEntries().forEach((entry) => {
        if (!entry.name.startsWith(CDN_CONFIG.baseUrl)) return;
        recordCdnMetric('resource_timing', entry.name, entry.duration);
      });
    }).observe({ type: 'resource', buffered: true });
  } catch {
    // PerformanceObserver not supported
  }
}

export function getCdnMetrics() {
  return [..._cdnMetrics];
}

// ── Initialise ────────────────────────────────────────────────────────────────

/**
 * Bootstrap CDN: rewrite asset URLs, start performance monitoring,
 * and detect the nearest region.
 */
export async function initCdn() {
  rewriteAssetUrls();
  observeCdnPerformance();
  const region = await detectNearestRegion();
  console.debug(`[cdn] nearest region: ${region}`);
  return region;
}
