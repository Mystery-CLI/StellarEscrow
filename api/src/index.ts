import { ApiClient } from './client';
import { TradesApi, EventsApi, BlockchainApi } from './resources';
import { ApiClientConfig } from './types';

export class EscrowApi {
  private client: ApiClient;
  public trades: TradesApi;
  public events: EventsApi;
  public blockchain: BlockchainApi;

  constructor(config: ApiClientConfig) {
    this.client = new ApiClient(config);
    this.trades = new TradesApi(this.client);
    this.events = new EventsApi(this.client);
    this.blockchain = new BlockchainApi(this.client);
  }

  addAuthToken(token: string) {
    this.client.addRequestInterceptor((config) => {
      config.headers.Authorization = `Bearer ${token}`;
      return config;
    });
  }

  addErrorHandler(handler: (error: any) => void) {
    this.client.addErrorInterceptor(async (error) => {
      handler(error);
      throw error;
    });
  }

  addResponseLogger() {
    this.client.addResponseInterceptor((response) => {
      console.log(`[API] ${response.config.method?.toUpperCase()} ${response.config.url} - ${response.status}`);
      return response;
    });
  }
}

export const createApi = (baseURL: string, mockEnabled = false): EscrowApi => {
  return new EscrowApi({
    baseURL,
    timeout: 30000,
    mockEnabled,
    retryConfig: {
      maxRetries: 3,
      delayMs: 1000,
      backoffMultiplier: 2,
    },
  });
};
