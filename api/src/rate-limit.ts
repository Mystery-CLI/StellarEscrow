/**
 * Rate Limiting Service for StellarEscrow API
 *
 * Features:
 *  - Sliding-window in-memory rate limiting
 *  - Pre-defined rate tiers (free / standard / premium / admin / public)
 *  - Per-identifier whitelist / blacklist support
 *  - Real-time monitoring with counters and snapshots
 *  - Notification hooks for limit-exceeded events
 *  - Axios request interceptor integration (client-side guard + header injection)
 */

// ── Types ─────────────────────────────────────────────────────────────────────

export type RateTier = 'public' | 'free' | 'standard' | 'premium' | 'admin';

export interface RateTierConfig {
  /** Maximum requests allowed within the window. */
  maxRequests: number;
  /** Sliding window duration in milliseconds. */
  windowMs: number;
  /** Human-readable label used in notifications / headers. */
  label: string;
}

export interface RateLimitEntry {
  /** Timestamps (epoch ms) of requests inside the current window. */
  timestamps: number[];
  /** Tier this key is pinned to (overrides the service default). */
  tier?: RateTier;
  /** Whether this key has been permanently blocked (blacklisted). */
  blocked: boolean;
}

export interface RateLimitStatus {
  key: string;
  tier: RateTier;
  allowed: boolean;
  remaining: number;
  limit: number;
  windowMs: number;
  resetAt: number;
  blacklisted: boolean;
  whitelisted: boolean;
}

export interface RateLimitNotification {
  type: 'exceeded' | 'blacklisted' | 'warning';
  key: string;
  tier: RateTier;
  timestamp: number;
  requestCount: number;
  limit: number;
  /** The notification fires as a "warning" when usage crosses this fraction (0–1). */
  usageFraction: number;
}

export type RateLimitNotificationHandler = (notification: RateLimitNotification) => void;

export interface RateLimitMonitorSnapshot {
  totalRequests: number;
  totalAllowed: number;
  totalBlocked: number;
  limitExceededCount: number;
  blacklistHits: number;
  whitelistHits: number;
  activeKeys: number;
  tierBreakdown: Record<RateTier, { requests: number; blocked: number }>;
  topKeys: Array<{ key: string; requests: number }>;
  timestamp: number;
}

export interface RateLimitServiceOptions {
  /** Default tier applied to new keys. Default: 'standard'. */
  defaultTier?: RateTier;
  /** Fraction (0–1) of the limit at which a warning notification fires. Default: 0.8. */
  warningThreshold?: number;
  /** Custom tier configurations merged on top of built-in defaults. */
  tiers?: Partial<Record<RateTier, RateTierConfig>>;
  /** Initial whitelist of keys (they bypass all limits). */
  whitelist?: string[];
  /** Initial blacklist of keys (all requests are rejected immediately). */
  blacklist?: string[];
  /** Notification handlers invoked on limit-exceeded / blacklist / warning events. */
  notificationHandlers?: RateLimitNotificationHandler[];
}

// ── Built-in tier definitions ─────────────────────────────────────────────────

export const DEFAULT_RATE_TIERS: Record<RateTier, RateTierConfig> = {
  /** Open endpoints — aggressively throttled to deter scraping. */
  public: { maxRequests: 20, windowMs: 60_000, label: 'Public' },
  /** Unauthenticated / registered-but-unverified users. */
  free: { maxRequests: 60, windowMs: 60_000, label: 'Free' },
  /** Authenticated standard users. */
  standard: { maxRequests: 200, windowMs: 60_000, label: 'Standard' },
  /** Power users / partners with elevated quotas. */
  premium: { maxRequests: 1_000, windowMs: 60_000, label: 'Premium' },
  /** Internal services / admin tools — essentially unlimited for normal usage. */
  admin: { maxRequests: 10_000, windowMs: 60_000, label: 'Admin' },
};

// ── Rate Limiting Service ─────────────────────────────────────────────────────

export class RateLimitService {
  private readonly tiers: Record<RateTier, RateTierConfig>;
  private readonly defaultTier: RateTier;
  private readonly warningThreshold: number;

  private readonly store: Map<string, RateLimitEntry> = new Map();
  private readonly whitelist: Set<string>;
  private readonly blacklist: Set<string>;
  private readonly handlers: RateLimitNotificationHandler[];

  // Monitoring counters
  private stats = {
    totalRequests: 0,
    totalAllowed: 0,
    totalBlocked: 0,
    limitExceeded: 0,
    blacklistHits: 0,
    whitelistHits: 0,
    tierRequests: {} as Record<RateTier, number>,
    tierBlocked: {} as Record<RateTier, number>,
    keyCounts: new Map<string, number>(),
  };

  constructor(options: RateLimitServiceOptions = {}) {
    this.defaultTier = options.defaultTier ?? 'standard';
    this.warningThreshold = options.warningThreshold ?? 0.8;
    this.tiers = { ...DEFAULT_RATE_TIERS, ...(options.tiers ?? {}) };
    this.whitelist = new Set(options.whitelist ?? []);
    this.blacklist = new Set(options.blacklist ?? []);
    this.handlers = [...(options.notificationHandlers ?? [])];

    // Initialise tier stat buckets
    for (const tier of Object.keys(this.tiers) as RateTier[]) {
      this.stats.tierRequests[tier] = 0;
      this.stats.tierBlocked[tier] = 0;
    }
  }

  // ── Core check ─────────────────────────────────────────────────────────────

  /**
   * Check whether a request from `key` is permitted.
   * Records the attempt in the sliding window and updates all counters.
   */
  check(key: string, tierOverride?: RateTier): RateLimitStatus {
    this.stats.totalRequests++;

    // Whitelist — always allow, skip window tracking
    if (this.whitelist.has(key)) {
      this.stats.totalAllowed++;
      this.stats.whitelistHits++;
      const tier = tierOverride ?? this.defaultTier;
      const cfg = this.tiers[tier];
      return {
        key,
        tier,
        allowed: true,
        remaining: cfg.maxRequests,
        limit: cfg.maxRequests,
        windowMs: cfg.windowMs,
        resetAt: Date.now() + cfg.windowMs,
        blacklisted: false,
        whitelisted: true,
      };
    }

    // Blacklist — always reject
    if (this.blacklist.has(key)) {
      this.stats.totalBlocked++;
      this.stats.blacklistHits++;
      const tier = tierOverride ?? this.defaultTier;
      const cfg = this.tiers[tier];
      this._notify({
        type: 'blacklisted',
        key,
        tier,
        timestamp: Date.now(),
        requestCount: 0,
        limit: cfg.maxRequests,
        usageFraction: 1,
      });
      return {
        key,
        tier,
        allowed: false,
        remaining: 0,
        limit: cfg.maxRequests,
        windowMs: cfg.windowMs,
        resetAt: Date.now() + cfg.windowMs,
        blacklisted: true,
        whitelisted: false,
      };
    }

    const now = Date.now();
    let entry = this.store.get(key);
    if (!entry) {
      entry = { timestamps: [], blocked: false };
      this.store.set(key, entry);
    }

    const effectiveTier: RateTier = tierOverride ?? entry.tier ?? this.defaultTier;
    const cfg = this.tiers[effectiveTier];

    // Slide the window: discard timestamps older than windowMs
    entry.timestamps = entry.timestamps.filter((t) => now - t < cfg.windowMs);

    this.stats.tierRequests[effectiveTier] = (this.stats.tierRequests[effectiveTier] ?? 0) + 1;
    this.stats.keyCounts.set(key, (this.stats.keyCounts.get(key) ?? 0) + 1);

    const count = entry.timestamps.length;
    const usageFraction = count / cfg.maxRequests;
    const resetAt = entry.timestamps.length > 0
      ? entry.timestamps[0] + cfg.windowMs
      : now + cfg.windowMs;

    // Warning notification (approaching the limit)
    if (usageFraction >= this.warningThreshold && usageFraction < 1) {
      this._notify({
        type: 'warning',
        key,
        tier: effectiveTier,
        timestamp: now,
        requestCount: count,
        limit: cfg.maxRequests,
        usageFraction,
      });
    }

    // Limit exceeded
    if (count >= cfg.maxRequests) {
      this.stats.totalBlocked++;
      this.stats.limitExceeded++;
      this.stats.tierBlocked[effectiveTier] = (this.stats.tierBlocked[effectiveTier] ?? 0) + 1;
      entry.blocked = true;
      this._notify({
        type: 'exceeded',
        key,
        tier: effectiveTier,
        timestamp: now,
        requestCount: count,
        limit: cfg.maxRequests,
        usageFraction: 1,
      });
      return { key, tier: effectiveTier, allowed: false, remaining: 0, limit: cfg.maxRequests, windowMs: cfg.windowMs, resetAt, blacklisted: false, whitelisted: false };
    }

    // Allowed
    entry.timestamps.push(now);
    entry.blocked = false;
    this.stats.totalAllowed++;
    return {
      key,
      tier: effectiveTier,
      allowed: true,
      remaining: cfg.maxRequests - entry.timestamps.length,
      limit: cfg.maxRequests,
      windowMs: cfg.windowMs,
      resetAt,
      blacklisted: false,
      whitelisted: false,
    };
  }

  /**
   * Query the current status for a key without recording an attempt.
   */
  peek(key: string, tierOverride?: RateTier): RateLimitStatus {
    const now = Date.now();
    const isBlacklisted = this.blacklist.has(key);
    const isWhitelisted = this.whitelist.has(key);
    const entry = this.store.get(key);
    const effectiveTier: RateTier = tierOverride ?? entry?.tier ?? this.defaultTier;
    const cfg = this.tiers[effectiveTier];
    const recentTimestamps = (entry?.timestamps ?? []).filter((t) => now - t < cfg.windowMs);
    const count = recentTimestamps.length;
    const resetAt = recentTimestamps.length > 0 ? recentTimestamps[0] + cfg.windowMs : now + cfg.windowMs;

    return {
      key,
      tier: effectiveTier,
      allowed: !isBlacklisted && (isWhitelisted || count < cfg.maxRequests),
      remaining: isWhitelisted ? cfg.maxRequests : Math.max(0, cfg.maxRequests - count),
      limit: cfg.maxRequests,
      windowMs: cfg.windowMs,
      resetAt,
      blacklisted: isBlacklisted,
      whitelisted: isWhitelisted,
    };
  }

  // ── Tier management ────────────────────────────────────────────────────────

  setTier(key: string, tier: RateTier): void {
    let entry = this.store.get(key);
    if (!entry) {
      entry = { timestamps: [], blocked: false };
      this.store.set(key, entry);
    }
    entry.tier = tier;
  }

  getTier(key: string): RateTier {
    return this.store.get(key)?.tier ?? this.defaultTier;
  }

  resetKey(key: string): void {
    const entry = this.store.get(key);
    if (entry) {
      entry.timestamps = [];
      entry.blocked = false;
    }
  }

  // ── Whitelist management ────────────────────────────────────────────────────

  addToWhitelist(key: string): void {
    this.whitelist.add(key);
    this.blacklist.delete(key); // mutually exclusive
  }

  removeFromWhitelist(key: string): void {
    this.whitelist.delete(key);
  }

  isWhitelisted(key: string): boolean {
    return this.whitelist.has(key);
  }

  getWhitelist(): string[] {
    return [...this.whitelist];
  }

  // ── Blacklist management ────────────────────────────────────────────────────

  addToBlacklist(key: string): void {
    this.blacklist.add(key);
    this.whitelist.delete(key); // mutually exclusive
  }

  removeFromBlacklist(key: string): void {
    this.blacklist.delete(key);
  }

  isBlacklisted(key: string): boolean {
    return this.blacklist.has(key);
  }

  getBlacklist(): string[] {
    return [...this.blacklist];
  }

  // ── Notification management ────────────────────────────────────────────────

  addNotificationHandler(handler: RateLimitNotificationHandler): void {
    this.handlers.push(handler);
  }

  removeNotificationHandler(handler: RateLimitNotificationHandler): void {
    const idx = this.handlers.indexOf(handler);
    if (idx !== -1) this.handlers.splice(idx, 1);
  }

  private _notify(notification: RateLimitNotification): void {
    for (const handler of this.handlers) {
      try {
        handler(notification);
      } catch {
        // Notification handlers must never crash the service
      }
    }
  }

  // ── Monitoring ─────────────────────────────────────────────────────────────

  getSnapshot(): RateLimitMonitorSnapshot {
    const tierBreakdown = {} as Record<RateTier, { requests: number; blocked: number }>;
    for (const tier of Object.keys(this.tiers) as RateTier[]) {
      tierBreakdown[tier] = {
        requests: this.stats.tierRequests[tier] ?? 0,
        blocked: this.stats.tierBlocked[tier] ?? 0,
      };
    }

    const topKeys = [...this.stats.keyCounts.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10)
      .map(([key, requests]) => ({ key, requests }));

    return {
      totalRequests: this.stats.totalRequests,
      totalAllowed: this.stats.totalAllowed,
      totalBlocked: this.stats.totalBlocked,
      limitExceededCount: this.stats.limitExceeded,
      blacklistHits: this.stats.blacklistHits,
      whitelistHits: this.stats.whitelistHits,
      activeKeys: this.store.size,
      tierBreakdown,
      topKeys,
      timestamp: Date.now(),
    };
  }

  resetStats(): void {
    this.stats.totalRequests = 0;
    this.stats.totalAllowed = 0;
    this.stats.totalBlocked = 0;
    this.stats.limitExceeded = 0;
    this.stats.blacklistHits = 0;
    this.stats.whitelistHits = 0;
    this.stats.keyCounts.clear();
    for (const tier of Object.keys(this.tiers) as RateTier[]) {
      this.stats.tierRequests[tier] = 0;
      this.stats.tierBlocked[tier] = 0;
    }
  }

  /**
   * Purge window data for all keys whose last request is older than `maxIdleMs`.
   * Call periodically to prevent unbounded memory growth.
   */
  evictStaleEntries(maxIdleMs = 5 * 60_000): number {
    const threshold = Date.now() - maxIdleMs;
    let evicted = 0;
    for (const [key, entry] of this.store) {
      const lastSeen = entry.timestamps.at(-1) ?? 0;
      if (lastSeen < threshold) {
        this.store.delete(key);
        evicted++;
      }
    }
    return evicted;
  }

  getTierConfig(tier: RateTier): RateTierConfig {
    return { ...this.tiers[tier] };
  }

  getAllTierConfigs(): Record<RateTier, RateTierConfig> {
    return { ...this.tiers };
  }
}

// ── Middleware factory ─────────────────────────────────────────────────────────

/**
 * Builds an Axios **request** interceptor that:
 *  1. Checks the rate limit for the resolved `key`.
 *  2. Injects `X-RateLimit-*` headers into the outgoing request so downstream
 *     servers / the browser console can observe quota state.
 *  3. Rejects the request immediately (before network I/O) if blocked.
 *
 * @param service  The `RateLimitService` instance to use.
 * @param keyResolver  Derive a per-request key from the Axios config.
 *                     Defaults to using `config.baseURL ?? 'default'`.
 * @param tierResolver Optional: derive the tier per-request.
 */
export function createRateLimitInterceptor(
  service: RateLimitService,
  keyResolver?: (config: any) => string,
  tierResolver?: (config: any) => RateTier | undefined,
): (config: any) => any {
  return (config: any) => {
    const key = keyResolver ? keyResolver(config) : (config.baseURL ?? 'default');
    const tier = tierResolver ? tierResolver(config) : undefined;
    const status = service.check(key, tier);

    // Inject rate limit headers (visible in DevTools / server logs)
    const headers: Record<string, string> = config.headers ?? {};
    headers['X-RateLimit-Limit'] = String(status.limit);
    headers['X-RateLimit-Remaining'] = String(status.remaining);
    headers['X-RateLimit-Reset'] = String(Math.ceil(status.resetAt / 1000));
    headers['X-RateLimit-Tier'] = status.tier;
    config.headers = headers;

    if (!status.allowed) {
      const reason = status.blacklisted ? 'blacklisted' : 'rate limit exceeded';
      const error: any = new Error(`[RateLimit] Request blocked for key "${key}": ${reason}`);
      error.code = 'RATE_LIMITED';
      error.rateLimitStatus = status;
      return Promise.reject(error);
    }

    return config;
  };
}

// ── Singleton helpers ──────────────────────────────────────────────────────────

let _defaultService: RateLimitService | undefined;

/**
 * Returns (or lazily creates) a module-level default `RateLimitService`.
 * Useful for quick setup without threading an instance through the call stack.
 */
export function getDefaultRateLimitService(options?: RateLimitServiceOptions): RateLimitService {
  if (!_defaultService) {
    _defaultService = new RateLimitService(options);
  }
  return _defaultService;
}

/** Reset the module-level singleton (primarily useful in tests). */
export function resetDefaultRateLimitService(): void {
  _defaultService = undefined;
}
