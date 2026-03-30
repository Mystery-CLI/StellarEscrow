/**
 * In-memory sliding-window rate limiter (per IP).
 */

import { RateLimitConfig } from './types';

interface Window {
  count: number;
  windowStart: number;
}

export class RateLimiter {
  private windows = new Map<string, Window>();

  constructor(private config: RateLimitConfig) {}

  /**
   * Check whether the IP is within its rate limit.
   * Returns { allowed, remaining, resetMs }.
   */
  check(ip: string): { allowed: boolean; remaining: number; resetMs: number } {
    const now = Date.now();
    const win = this.windows.get(ip);

    if (!win || now - win.windowStart >= this.config.windowMs) {
      // New window
      this.windows.set(ip, { count: 1, windowStart: now });
      return {
        allowed: true,
        remaining: this.config.maxRequests - 1,
        resetMs: this.config.windowMs,
      };
    }

    const resetMs = this.config.windowMs - (now - win.windowStart);

    if (win.count >= this.config.maxRequests) {
      return { allowed: false, remaining: 0, resetMs };
    }

    win.count++;
    return { allowed: true, remaining: this.config.maxRequests - win.count, resetMs };
  }

  /** Evict stale windows to prevent unbounded memory growth. */
  evict(): void {
    const now = Date.now();
    for (const [ip, win] of this.windows) {
      if (now - win.windowStart >= this.config.windowMs * 2) {
        this.windows.delete(ip);
      }
    }
  }
}
