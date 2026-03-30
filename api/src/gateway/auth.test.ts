import { authenticate } from './auth';
import { GatewayConfig } from './types';

const config: GatewayConfig = {
  port: 4000,
  apiKeys: ['api-key-abc'],
  adminKeys: ['admin-key-xyz'],
  upstreams: [],
  rateLimit: { windowMs: 60_000, maxRequests: 60 },
};

describe('authenticate', () => {
  it('allows public paths without a key', () => {
    const r = authenticate(undefined, undefined, '/health', config);
    expect(r.authenticated).toBe(true);
  });

  it('rejects missing key on protected path', () => {
    const r = authenticate(undefined, undefined, '/events', config);
    expect(r.authenticated).toBe(false);
    expect(r.reason).toMatch(/missing/i);
  });

  it('accepts valid API key via Authorization header', () => {
    const r = authenticate('Bearer api-key-abc', undefined, '/events', config);
    expect(r.authenticated).toBe(true);
    expect(r.role).toBe('api');
  });

  it('accepts valid API key via X-Api-Key header', () => {
    const r = authenticate(undefined, 'api-key-abc', '/events', config);
    expect(r.authenticated).toBe(true);
    expect(r.role).toBe('api');
  });

  it('accepts admin key and grants admin role', () => {
    const r = authenticate('Bearer admin-key-xyz', undefined, '/events', config);
    expect(r.authenticated).toBe(true);
    expect(r.role).toBe('admin');
  });

  it('rejects API key on admin path', () => {
    const r = authenticate('Bearer api-key-abc', undefined, '/admin/users', config);
    expect(r.authenticated).toBe(false);
    expect(r.reason).toMatch(/admin/i);
  });

  it('allows admin key on admin path', () => {
    const r = authenticate('Bearer admin-key-xyz', undefined, '/admin/users', config);
    expect(r.authenticated).toBe(true);
    expect(r.role).toBe('admin');
  });

  it('rejects invalid key', () => {
    const r = authenticate('Bearer wrong-key', undefined, '/events', config);
    expect(r.authenticated).toBe(false);
    expect(r.reason).toMatch(/invalid/i);
  });

  it('rejects expired JWT', () => {
    const cfg = { ...config, jwtSecret: 'secret' };
    // Build a JWT with exp in the past
    const header = Buffer.from(JSON.stringify({ alg: 'HS256' })).toString('base64url');
    const payload = Buffer.from(JSON.stringify({ exp: 1 })).toString('base64url');
    const token = `${header}.${payload}.sig`;
    const r = authenticate(`Bearer ${token}`, undefined, '/events', cfg);
    expect(r.authenticated).toBe(false);
    expect(r.reason).toMatch(/expired/i);
  });
});
