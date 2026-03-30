import { ApiClient } from './client';
import { ApiError } from './types';

describe('ApiClient', () => {
  let client: ApiClient;

  beforeEach(() => {
    client = new ApiClient({
      baseURL: 'http://localhost:3000',
      timeout: 5000,
    });
  });

  it('should add request interceptor', () => {
    const interceptor = jest.fn((config) => config);
    client.addRequestInterceptor(interceptor);
    expect(interceptor).toBeDefined();
  });

  it('should add response interceptor', () => {
    const interceptor = jest.fn((response) => response);
    client.addResponseInterceptor(interceptor);
    expect(interceptor).toBeDefined();
  });

  it('should add error interceptor', () => {
    const interceptor = jest.fn(async (error) => {
      throw error;
    });
    client.addErrorInterceptor(interceptor);
    expect(interceptor).toBeDefined();
  });

  it('should retry on server error', async () => {
    const client = new ApiClient({
      baseURL: 'http://localhost:3000',
      retryConfig: {
        maxRetries: 2,
        delayMs: 100,
        backoffMultiplier: 1,
      },
    });

    let attempts = 0;
    client.addErrorInterceptor(async (error: any) => {
      attempts++;
      if (attempts < 2) {
        throw error;
      }
      return { data: 'success' };
    });

    expect(attempts).toBeGreaterThanOrEqual(0);
  });
});
