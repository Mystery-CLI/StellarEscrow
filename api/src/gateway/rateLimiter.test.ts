import { RateLimiter } from './rateLimiter';

describe('RateLimiter', () => {
  it('allows requests within the limit', () => {
    const rl = new RateLimiter({ windowMs: 60_000, maxRequests: 3 });
    expect(rl.check('1.2.3.4').allowed).toBe(true);
    expect(rl.check('1.2.3.4').allowed).toBe(true);
    expect(rl.check('1.2.3.4').allowed).toBe(true);
  });

  it('blocks requests over the limit', () => {
    const rl = new RateLimiter({ windowMs: 60_000, maxRequests: 2 });
    rl.check('1.2.3.4');
    rl.check('1.2.3.4');
    const result = rl.check('1.2.3.4');
    expect(result.allowed).toBe(false);
    expect(result.remaining).toBe(0);
  });

  it('tracks different IPs independently', () => {
    const rl = new RateLimiter({ windowMs: 60_000, maxRequests: 1 });
    expect(rl.check('1.1.1.1').allowed).toBe(true);
    expect(rl.check('2.2.2.2').allowed).toBe(true);
    expect(rl.check('1.1.1.1').allowed).toBe(false);
  });

  it('resets after the window expires', () => {
    const rl = new RateLimiter({ windowMs: 1, maxRequests: 1 });
    rl.check('1.2.3.4');
    return new Promise<void>((resolve) =>
      setTimeout(() => {
        expect(rl.check('1.2.3.4').allowed).toBe(true);
        resolve();
      }, 5)
    );
  });

  it('returns correct remaining count', () => {
    const rl = new RateLimiter({ windowMs: 60_000, maxRequests: 5 });
    expect(rl.check('1.2.3.4').remaining).toBe(4);
    expect(rl.check('1.2.3.4').remaining).toBe(3);
  });
});
