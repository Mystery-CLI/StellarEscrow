/**
 * Rate Limiting Service — pure self-contained test harness
 *
 * No test runner, no npm, no internet required.
 * Uses only vanilla TypeScript / JavaScript primitives.
 *
 * How to run (when Node is available):
 *   ts-node rate-limit.test.ts
 *   — OR —
 *   tsc rate-limit.test.ts --outDir /tmp && node /tmp/rate-limit.test.js
 *
 * The file is also valid Jest spec syntax so it runs normally under Jest
 * when node_modules are installed later.
 */

import {
  RateLimitService,
  DEFAULT_RATE_TIERS,
  createRateLimitInterceptor,
  getDefaultRateLimitService,
  resetDefaultRateLimitService,
  RateLimitNotification,
  RateTier,
  RateLimitMonitorSnapshot,
} from './rate-limit';

// ─────────────────────────────────────────────────────────────────────────────
// Minimal jest-compatible shim so the file runs outside Jest too
// ─────────────────────────────────────────────────────────────────────────────

declare const describe: Function | undefined;
declare const it: Function | undefined;
declare const expect: Function | undefined;
declare const beforeEach: Function | undefined;
declare const afterEach: Function | undefined;

const _isJest =
  typeof describe !== 'undefined' &&
  typeof it !== 'undefined' &&
  typeof expect !== 'undefined';

/** Lightweight test state for the standalone harness */
interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
}

const _results: TestResult[] = [];
let _currentSuite = '';

/* eslint-disable @typescript-eslint/no-explicit-any */
const _describe = (label: string, fn: () => void) => {
  if (_isJest) return (globalThis as any).describe(label, fn);
  _currentSuite = label;
  fn();
};

const _it = (label: string, fn: () => void | Promise<void>) => {
  if (_isJest) return (globalThis as any).it(label, fn);
  const name = `${_currentSuite} › ${label}`;
  try {
    const result = fn();
    if (result instanceof Promise) {
      result
        .then(() => _results.push({ name, passed: true }))
        .catch((err: unknown) => _results.push({ name, passed: false, error: String(err) }));
    } else {
      _results.push({ name, passed: true });
    }
  } catch (err) {
    _results.push({ name, passed: false, error: String(err) });
  }
};

const _beforeEach = (fn: () => void) => {
  if (_isJest) return (globalThis as any).beforeEach(fn);
  // In standalone mode beforeEach is called manually inside suites below
};

const _afterEach = (fn: () => void) => {
  if (_isJest) return (globalThis as any).afterEach(fn);
};

function _assert(condition: boolean, message?: string): void {
  if (!condition) throw new Error(`AssertionError: ${message ?? 'expected truthy'}`);
}

function _assertEq<T>(actual: T, expected: T, message?: string): void {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a !== e) throw new Error(`AssertionError: ${message ?? `expected ${e} but got ${a}`}`);
}

function _assertContains<T>(arr: T[], item: T): void {
  if (!arr.some((x) => JSON.stringify(x) === JSON.stringify(item))) {
    throw new Error(`AssertionError: array does not contain ${JSON.stringify(item)}`);
  }
}

const _expect = (actual: unknown) => {
  if (_isJest) return (globalThis as any).expect(actual);
  return {
    toBe: (expected: unknown) => _assertEq(actual, expected),
    toEqual: (expected: unknown) => _assertEq(JSON.stringify(actual), JSON.stringify(expected)),
    toBeDefined: () => _assert(actual !== undefined && actual !== null, 'expected defined'),
    toBeGreaterThan: (n: number) => _assert((actual as number) > n, `${actual} > ${n}`),
    toBeGreaterThanOrEqual: (n: number) => _assert((actual as number) >= n, `${actual} >= ${n}`),
    toHaveLength: (n: number) => _assert((actual as any[]).length === n, `length ${(actual as any[]).length} === ${n}`),
    not: {
      toBe: (expected: unknown) => _assert(actual !== expected, `expected ${actual} !== ${expected}`),
      toThrow: () => { /* handled where called */ },
    },
    toMatchObject: (obj: Record<string, unknown>) => {
      for (const [k, v] of Object.entries(obj)) {
        _assertEq((actual as any)[k], v, `key "${k}"`);
      }
    },
    arrayContaining: (subset: unknown[]) => subset, // passthrough for standalone
  };
};

// Rejects helper for async middleware tests
async function _rejects(promise: Promise<unknown>, matcher?: Record<string, unknown>): Promise<void> {
  try {
    await promise;
    throw new Error('Expected promise to reject but it resolved.');
  } catch (err: unknown) {
    if ((err as any)?.message?.includes('Expected promise to reject')) throw err;
    if (matcher) {
      for (const [k, v] of Object.entries(matcher)) {
        _assertEq((err as any)[k], v, `rejection key "${k}"`);
      }
    }
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

function makeService(overrides: Partial<ConstructorParameters<typeof RateLimitService>[0]> = {}) {
  return new RateLimitService({
    defaultTier: 'free',
    tiers: {
      free:     { maxRequests: 3,     windowMs: 60_000, label: 'Free' },
      standard: { maxRequests: 10,    windowMs: 60_000, label: 'Standard' },
      premium:  { maxRequests: 50,    windowMs: 60_000, label: 'Premium' },
      admin:    { maxRequests: 1_000, windowMs: 60_000, label: 'Admin' },
      public:   { maxRequests: 2,     windowMs: 60_000, label: 'Public' },
    },
    ...overrides,
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test suites
// ─────────────────────────────────────────────────────────────────────────────

_describe('RateLimitService – core sliding window', () => {
  _it('allows requests within the limit', () => {
    const svc = makeService();
    _expect(svc.check('user-a').allowed).toBe(true);
    _expect(svc.check('user-a').allowed).toBe(true);
    _expect(svc.check('user-a').allowed).toBe(true);
  });

  _it('blocks the next request after the limit is reached', () => {
    const svc = makeService();
    svc.check('user-b'); svc.check('user-b'); svc.check('user-b');
    const r = svc.check('user-b');
    _expect(r.allowed).toBe(false);
    _expect(r.remaining).toBe(0);
  });

  _it('calculates remaining correctly', () => {
    const svc = makeService();
    _expect(svc.check('user-c').remaining).toBe(2);
    svc.check('user-c');
    _expect(svc.check('user-c').remaining).toBe(0);
  });

  _it('returns correct limit and windowMs in status', () => {
    const svc = makeService();
    const s = svc.check('user-d');
    _expect(s.limit).toBe(3);
    _expect(s.windowMs).toBe(60_000);
  });

  _it('tracks different keys independently', () => {
    const svc = makeService();
    svc.check('key-1'); svc.check('key-1'); svc.check('key-1');
    _expect(svc.check('key-1').allowed).toBe(false);
    _expect(svc.check('key-2').allowed).toBe(true);
  });

  _it('resetKey() restores access', () => {
    const svc = makeService();
    svc.check('user-e'); svc.check('user-e'); svc.check('user-e');
    _expect(svc.check('user-e').allowed).toBe(false);
    svc.resetKey('user-e');
    _expect(svc.check('user-e').allowed).toBe(true);
  });

  _it('slides the window: old timestamps are pruned', () => {
    const svc = new RateLimitService({
      defaultTier: 'free',
      tiers: { free: { maxRequests: 2, windowMs: 100, label: 'Free' } } as any,
    });
    svc.check('slide-key'); svc.check('slide-key');
    // Backdate timestamps so they are outside the 100 ms window
    const entry = (svc as any).store.get('slide-key');
    entry.timestamps = entry.timestamps.map(() => Date.now() - 200);
    _expect(svc.check('slide-key').allowed).toBe(true);
  });
});

_describe('RateLimitService – rate tiers', () => {
  _it('exposes all five default tiers', () => {
    const tiers = Object.keys(DEFAULT_RATE_TIERS) as RateTier[];
    for (const t of ['public', 'free', 'standard', 'premium', 'admin'] as RateTier[]) {
      _assert(tiers.includes(t), `missing tier ${t}`);
    }
  });

  _it('uses defaultTier when no per-key tier is set', () => {
    const svc = makeService({ defaultTier: 'standard' });
    const s = svc.check('user-f');
    _expect(s.tier).toBe('standard');
    _expect(s.limit).toBe(10);
  });

  _it('honours a tier override passed to check()', () => {
    const svc = makeService();
    const s = svc.check('user-g', 'admin');
    _expect(s.tier).toBe('admin');
    _expect(s.limit).toBe(1_000);
  });

  _it('respects a per-key tier set via setTier()', () => {
    const svc = makeService();
    svc.setTier('user-h', 'premium');
    const s = svc.check('user-h');
    _expect(s.tier).toBe('premium');
    _expect(s.limit).toBe(50);
  });

  _it('getTier() returns the assigned tier', () => {
    const svc = makeService();
    svc.setTier('user-i', 'admin');
    _expect(svc.getTier('user-i')).toBe('admin');
  });

  _it('getTierConfig() returns a copy (mutations do not affect service)', () => {
    const svc = makeService();
    const cfg = svc.getTierConfig('free');
    _expect(cfg.maxRequests).toBe(3);
    cfg.maxRequests = 999;
    _expect(svc.getTierConfig('free').maxRequests).toBe(3);
  });

  _it('getAllTierConfigs() returns all five tiers', () => {
    const svc = makeService();
    _expect(Object.keys(svc.getAllTierConfigs()).length).toBe(5);
  });
});

_describe('RateLimitService – whitelist', () => {
  _it('whitelisted keys bypass the limit', () => {
    const svc = makeService({ whitelist: ['always-ok'] });
    for (let i = 0; i < 100; i++) _expect(svc.check('always-ok').allowed).toBe(true);
  });

  _it('whitelisted flag is set in status', () => {
    const svc = makeService();
    svc.addToWhitelist('wl-key');
    const s = svc.check('wl-key');
    _expect(s.whitelisted).toBe(true);
    _expect(s.blacklisted).toBe(false);
  });

  _it('addToWhitelist removes from blacklist', () => {
    const svc = makeService({ blacklist: ['dual-key'] });
    svc.addToWhitelist('dual-key');
    _expect(svc.isBlacklisted('dual-key')).toBe(false);
    _expect(svc.isWhitelisted('dual-key')).toBe(true);
  });

  _it('removeFromWhitelist re-applies rate limiting', () => {
    const svc = makeService();
    svc.addToWhitelist('temp-wl');
    svc.removeFromWhitelist('temp-wl');
    svc.check('temp-wl'); svc.check('temp-wl'); svc.check('temp-wl');
    _expect(svc.check('temp-wl').allowed).toBe(false);
  });

  _it('getWhitelist() returns all whitelisted keys', () => {
    const svc = makeService({ whitelist: ['a', 'b'] });
    svc.addToWhitelist('c');
    const wl = svc.getWhitelist();
    _assert(wl.includes('a') && wl.includes('b') && wl.includes('c'));
  });

  _it('whitelist hits are counted in snapshot', () => {
    const svc = makeService({ whitelist: ['wl-stat'] });
    svc.check('wl-stat'); svc.check('wl-stat');
    _expect(svc.getSnapshot().whitelistHits).toBe(2);
  });
});

_describe('RateLimitService – blacklist', () => {
  _it('blacklisted keys are always blocked', () => {
    const svc = makeService({ blacklist: ['bad-actor'] });
    _expect(svc.check('bad-actor').allowed).toBe(false);
  });

  _it('blacklisted flag is set in status', () => {
    const svc = makeService();
    svc.addToBlacklist('bl-key');
    const s = svc.check('bl-key');
    _expect(s.blacklisted).toBe(true);
    _expect(s.whitelisted).toBe(false);
  });

  _it('addToBlacklist removes from whitelist', () => {
    const svc = makeService({ whitelist: ['flip-key'] });
    svc.addToBlacklist('flip-key');
    _expect(svc.isWhitelisted('flip-key')).toBe(false);
    _expect(svc.isBlacklisted('flip-key')).toBe(true);
  });

  _it('removeFromBlacklist restores access', () => {
    const svc = makeService({ blacklist: ['temp-bl'] });
    svc.removeFromBlacklist('temp-bl');
    _expect(svc.check('temp-bl').allowed).toBe(true);
  });

  _it('getBlacklist() returns all blacklisted keys', () => {
    const svc = makeService({ blacklist: ['x', 'y'] });
    svc.addToBlacklist('z');
    const bl = svc.getBlacklist();
    _assert(bl.includes('x') && bl.includes('y') && bl.includes('z'));
  });

  _it('blacklist hits are counted in snapshot', () => {
    const svc = makeService({ blacklist: ['bl-stat'] });
    svc.check('bl-stat'); svc.check('bl-stat');
    _expect(svc.getSnapshot().blacklistHits).toBe(2);
  });
});

_describe('RateLimitService – notifications', () => {
  _it('fires "exceeded" when limit is hit', () => {
    const received: RateLimitNotification[] = [];
    const svc = makeService({ notificationHandlers: [(n) => received.push(n)] });
    svc.check('n1'); svc.check('n1'); svc.check('n1');
    svc.check('n1'); // 4th — exceeded
    _assert(received.some((n) => n.type === 'exceeded'));
  });

  _it('fires "blacklisted" for blacklisted keys', () => {
    const received: RateLimitNotification[] = [];
    const svc = makeService({
      blacklist: ['blist-notif'],
      notificationHandlers: [(n) => received.push(n)],
    });
    svc.check('blist-notif');
    _assert(received.some((n) => n.type === 'blacklisted'));
  });

  _it('fires "warning" when threshold is crossed', () => {
    const received: RateLimitNotification[] = [];
    const svc = makeService({
      warningThreshold: 0.5,
      notificationHandlers: [(n) => received.push(n)],
    });
    svc.check('warn-key');
    svc.check('warn-key'); // 2/3 = 0.667 ≥ 0.5
    _assert(received.some((n) => n.type === 'warning'));
  });

  _it('notification payload contains key and tier', () => {
    const received: RateLimitNotification[] = [];
    const svc = makeService({ notificationHandlers: [(n) => received.push(n)] });
    svc.setTier('notif-tier-key', 'premium');
    for (let i = 0; i <= 50; i++) svc.check('notif-tier-key');
    const exceeded = received.find((n) => n.type === 'exceeded');
    _expect(exceeded?.key).toBe('notif-tier-key');
    _expect(exceeded?.tier).toBe('premium');
  });

  _it('addNotificationHandler() registers a new handler', () => {
    const svc = makeService();
    const events: RateLimitNotification[] = [];
    svc.addNotificationHandler((n) => events.push(n));
    svc.addToBlacklist('dyn-key');
    svc.check('dyn-key');
    _expect(events.length).toBeGreaterThan(0);
  });

  _it('removeNotificationHandler() deregisters a handler', () => {
    const svc = makeService();
    const events: RateLimitNotification[] = [];
    const handler = (n: RateLimitNotification) => events.push(n);
    svc.addNotificationHandler(handler);
    svc.removeNotificationHandler(handler);
    svc.addToBlacklist('removed-key');
    svc.check('removed-key');
    _expect(events.length).toBe(0);
  });

  _it('a throwing handler does not crash the service', () => {
    const svc = makeService({
      notificationHandlers: [() => { throw new Error('boom'); }],
    });
    svc.addToBlacklist('throw-key');
    let threw = false;
    try { svc.check('throw-key'); } catch { threw = true; }
    _expect(threw).toBe(false);
  });
});

_describe('RateLimitService – monitoring', () => {
  _it('snapshot reflects request totals', () => {
    const svc = makeService();
    svc.check('m1'); svc.check('m1'); svc.check('m2');
    const s = svc.getSnapshot();
    _expect(s.totalRequests).toBe(3);
    _expect(s.totalAllowed).toBe(3);
    _expect(s.totalBlocked).toBe(0);
  });

  _it('snapshot counts blocked requests', () => {
    const svc = makeService();
    svc.check('bl'); svc.check('bl'); svc.check('bl'); svc.check('bl');
    const s = svc.getSnapshot();
    _expect(s.totalBlocked).toBe(1);
    _expect(s.limitExceededCount).toBe(1);
  });

  _it('snapshot tierBreakdown matches requests per tier', () => {
    const svc = makeService({ defaultTier: 'standard' });
    svc.check('td-1', 'admin'); svc.check('td-2', 'admin');
    svc.check('td-3', 'free');
    const s = svc.getSnapshot();
    _expect(s.tierBreakdown.admin.requests).toBe(2);
    _expect(s.tierBreakdown.free.requests).toBe(1);
  });

  _it('snapshot.topKeys is sorted by request count descending', () => {
    const svc = makeService();
    svc.check('heavy'); svc.check('heavy'); svc.check('light');
    const s = svc.getSnapshot();
    _expect(s.topKeys[0].key).toBe('heavy');
    _expect(s.topKeys[0].requests).toBe(2);
  });

  _it('snapshot.activeKeys reflects the store size', () => {
    const svc = makeService();
    svc.check('key-a'); svc.check('key-b');
    _expect(svc.getSnapshot().activeKeys).toBe(2);
  });

  _it('resetStats() clears all counters', () => {
    const svc = makeService();
    svc.check('stat-key'); svc.check('stat-key'); svc.check('stat-key'); svc.check('stat-key');
    svc.resetStats();
    const s = svc.getSnapshot();
    _expect(s.totalRequests).toBe(0);
    _expect(s.totalBlocked).toBe(0);
    _expect(s.limitExceededCount).toBe(0);
  });

  _it('snapshot includes a timestamp', () => {
    const before = Date.now();
    _expect(makeService().getSnapshot().timestamp).toBeGreaterThanOrEqual(before);
  });
});

_describe('RateLimitService – peek()', () => {
  _it('peek does not count as a request', () => {
    const svc = makeService();
    svc.check('peek-key'); svc.check('peek-key'); // 2/3 used
    svc.peek('peek-key');                          // non-recording
    _expect(svc.check('peek-key').allowed).toBe(true);  // 3rd — OK
    _expect(svc.check('peek-key').allowed).toBe(false); // 4th — blocked
  });

  _it('peek returns blacklisted=true for blocked keys', () => {
    _expect(makeService({ blacklist: ['peek-bl'] }).peek('peek-bl').blacklisted).toBe(true);
  });

  _it('peek returns whitelisted=true for allowlisted keys', () => {
    _expect(makeService({ whitelist: ['peek-wl'] }).peek('peek-wl').whitelisted).toBe(true);
  });
});

_describe('RateLimitService – evictStaleEntries()', () => {
  _it('removes entries older than maxIdleMs', () => {
    const svc = makeService();
    svc.check('stale-key');
    const entry = (svc as any).store.get('stale-key');
    entry.timestamps = [Date.now() - 10 * 60_000]; // 10 min old
    const evicted = svc.evictStaleEntries(5 * 60_000); // 5 min threshold
    _expect(evicted).toBe(1);
    _expect(svc.getSnapshot().activeKeys).toBe(0);
  });

  _it('keeps fresh entries', () => {
    const svc = makeService();
    svc.check('fresh-key');
    _expect(svc.evictStaleEntries(60_000)).toBe(0);
    _expect(svc.getSnapshot().activeKeys).toBe(1);
  });
});

_describe('createRateLimitInterceptor()', () => {
  _it('injects X-RateLimit-* headers on allowed requests', () => {
    const svc = makeService();
    const interceptor = createRateLimitInterceptor(svc, () => 'test-key');
    const result = interceptor({ headers: {} });
    _assert(!_isJest || result !== undefined);
    _expect(result['X-RateLimit-Limit']).toBeDefined();
    _expect(result['X-RateLimit-Remaining']).toBeDefined();
    _expect(result['X-RateLimit-Reset']).toBeDefined();
    _expect(result['X-RateLimit-Tier']).toBeDefined();
  });

  _it('rejects for exceeded limit', async () => {
    const svc = makeService();
    const interceptor = createRateLimitInterceptor(svc, () => 'rl-key');
    const cfg = { headers: {} };
    interceptor(cfg); interceptor(cfg); interceptor(cfg);
    await _rejects(Promise.resolve(interceptor(cfg)), { code: 'RATE_LIMITED' });
  });

  _it('rejects for blacklisted keys', async () => {
    const svc = makeService({ blacklist: ['bl-intercept'] });
    const interceptor = createRateLimitInterceptor(svc, () => 'bl-intercept');
    await _rejects(Promise.resolve(interceptor({ headers: {} })), { code: 'RATE_LIMITED' });
  });

  _it('uses tierResolver when provided', () => {
    const svc = makeService();
    const interceptor = createRateLimitInterceptor(svc, () => 'tier-key', () => 'admin');
    const result = interceptor({ headers: {} });
    _expect(result['X-RateLimit-Tier']).toBe('admin');
  });

  _it('attaches rateLimitStatus to rejected errors', async () => {
    const svc = makeService();
    const interceptor = createRateLimitInterceptor(svc, () => 'rls-key');
    const cfg = { headers: {} };
    interceptor(cfg); interceptor(cfg); interceptor(cfg);
    let caught: any;
    try { await Promise.resolve(interceptor(cfg)); } catch (e) { caught = e; }
    _expect(caught?.rateLimitStatus?.allowed).toBe(false);
  });
});

_describe('Singleton helpers', () => {
  _it('returns the same instance on successive calls', () => {
    resetDefaultRateLimitService();
    const a = getDefaultRateLimitService();
    const b = getDefaultRateLimitService();
    _assert(a === b, 'singleton must return same instance');
    resetDefaultRateLimitService();
  });

  _it('creates a fresh instance after reset', () => {
    resetDefaultRateLimitService();
    const a = getDefaultRateLimitService();
    resetDefaultRateLimitService();
    const b = getDefaultRateLimitService();
    _assert(a !== b, 'should be a different instance after reset');
    resetDefaultRateLimitService();
  });

  _it('accepts whitelist option on first call', () => {
    resetDefaultRateLimitService();
    const svc = getDefaultRateLimitService({ whitelist: ['init-key'] });
    _expect(svc.isWhitelisted('init-key')).toBe(true);
    resetDefaultRateLimitService();
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Standalone runner: prints report when executed directly (not via Jest)
// ─────────────────────────────────────────────────────────────────────────────

if (!_isJest) {
  // Allow micro-tasks (async tests) to settle
  Promise.resolve().then(() => {
    setTimeout(() => {
      const passed = _results.filter((r) => r.passed).length;
      const failed = _results.filter((r) => !r.passed);
      console.log('\n══════════════════════════════════════════════════');
      console.log(` Rate Limit Service — Test Report`);
      console.log('══════════════════════════════════════════════════');
      for (const r of _results) {
        const icon = r.passed ? '✓' : '✗';
        console.log(`  ${icon} ${r.name}${r.error ? `\n      → ${r.error}` : ''}`);
      }
      console.log('──────────────────────────────────────────────────');
      console.log(` ${passed} passed, ${failed.length} failed out of ${_results.length} tests`);
      console.log('══════════════════════════════════════════════════\n');
      if (failed.length > 0) process.exit(1);
    }, 100);
  });
}
