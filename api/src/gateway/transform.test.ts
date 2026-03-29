import { transformRequest, transformResponse } from './transform';

describe('transformRequest', () => {
  it('converts camelCase to snake_case for trade paths', () => {
    const result = transformRequest('/trades', { tradeId: '1', buyerAddress: 'GABC' }) as any;
    expect(result.trade_id).toBe('1');
    expect(result.buyer_address).toBe('GABC');
  });

  it('passes through non-trade paths unchanged', () => {
    const body = { someKey: 'value' };
    expect(transformRequest('/events', body)).toBe(body);
  });

  it('handles null/undefined body', () => {
    expect(transformRequest('/trades', null)).toBeNull();
    expect(transformRequest('/trades', undefined)).toBeUndefined();
  });
});

describe('transformResponse', () => {
  it('wraps successful responses in an envelope', () => {
    const result = transformResponse({ id: 1 }, 200, 'indexer', 42) as any;
    expect(result.data).toEqual({ id: 1 });
    expect(result.meta.upstream).toBe('indexer');
    expect(result.meta.latencyMs).toBe(42);
    expect(result.meta.timestamp).toBeDefined();
  });

  it('passes through error responses without wrapping', () => {
    const body = { error: 'Not found' };
    expect(transformResponse(body, 404, 'indexer', 10)).toBe(body);
  });
});
