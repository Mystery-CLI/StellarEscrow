import axios from 'axios';
import { ApiClient } from './client';

const mockAxiosInstance = {
  interceptors: {
    request: { use: jest.fn() },
    response: { use: jest.fn() },
  },
  get: jest.fn(),
  post: jest.fn(),
  patch: jest.fn(),
  delete: jest.fn(),
};

jest.mock('axios', () => ({
  __esModule: true,
  default: {
    create: jest.fn(() => mockAxiosInstance),
  },
}));

describe('ApiClient unit', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockAxiosInstance.interceptors.request.use.mockClear();
    mockAxiosInstance.interceptors.response.use.mockClear();
  });

  it('applies request interceptors in registration order', () => {
    const client = new ApiClient({ baseURL: 'http://localhost:3000/api', timeout: 1000 });
    client.addRequestInterceptor((config) => {
      config.headers = { ...(config.headers ?? {}), 'x-trace-id': 'trace-1' };
      return config;
    });
    client.addRequestInterceptor((config) => {
      config.headers = { ...(config.headers ?? {}), authorization: 'Bearer token' };
      return config;
    });

    const requestHandler = mockAxiosInstance.interceptors.request.use.mock.calls[0][0];
    const config = requestHandler({ headers: {} });

    expect(config.headers).toMatchObject({
      'x-trace-id': 'trace-1',
      authorization: 'Bearer token',
    });
  });

  it('applies response interceptors in registration order', () => {
    const client = new ApiClient({ baseURL: 'http://localhost:3000/api' });
    client.addResponseInterceptor((response) => ({
      ...response,
      data: { ...response.data, first: true },
    }));
    client.addResponseInterceptor((response) => ({
      ...response,
      data: { ...response.data, second: true },
    }));

    const responseHandler = mockAxiosInstance.interceptors.response.use.mock.calls[0][0];
    const response = responseHandler({ data: { ok: true } });

    expect(response.data).toEqual({ ok: true, first: true, second: true });
  });

  it('retries transient server errors before succeeding', async () => {
    const client = new ApiClient({
      baseURL: 'http://localhost:3000/api',
      retryConfig: { maxRetries: 2, delayMs: 0, backoffMultiplier: 1 },
    });

    mockAxiosInstance.get
      .mockRejectedValueOnce({ response: { status: 503 }, message: 'service unavailable' })
      .mockResolvedValueOnce({ data: { ok: true } });

    await expect(client.get('/trades')).resolves.toEqual({ ok: true });
    expect(mockAxiosInstance.get).toHaveBeenCalledTimes(2);
  });

  it('parses terminal HTTP failures into ApiError objects', async () => {
    new ApiClient({ baseURL: 'http://localhost:3000/api' });
    const error = {
      code: 'ERR_BAD_REQUEST',
      message: 'Request failed with status code 404',
      response: { status: 404, data: { error: 'missing' } },
    } as any;

    const errorHandler = mockAxiosInstance.interceptors.response.use.mock.calls[0][1];
    await expect(errorHandler(error)).rejects.toMatchObject({
      code: 'ERR_BAD_REQUEST',
      message: 'Request failed with status code 404',
      status: 404,
      details: { error: 'missing' },
    });
  });

  it('lets error interceptors recover a failed request', async () => {
    const client = new ApiClient({
      baseURL: 'http://localhost:3000/api',
      retryConfig: { maxRetries: 0, delayMs: 0, backoffMultiplier: 1 },
    });

    client.addErrorInterceptor(async () => ({ data: { recovered: true } }));
    const errorHandler = mockAxiosInstance.interceptors.response.use.mock.calls[0][1];

    await expect(errorHandler(new Error('boom'))).resolves.toEqual({ data: { recovered: true } });
  });

  it('creates the underlying axios instance with the configured base URL', () => {
    new ApiClient({ baseURL: 'http://localhost:3000/api', timeout: 2500 });

    expect((axios as unknown as { create: jest.Mock }).create).toHaveBeenCalledWith({
      baseURL: 'http://localhost:3000/api',
      timeout: 2500,
    });
  });
});
