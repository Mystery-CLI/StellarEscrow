# API Integration Layer

Robust API client library with interceptors, retry logic, error handling, and mocking.

## Features

### ✅ API Client Library
- Axios-based HTTP client
- Type-safe with TypeScript
- Configurable base URL and timeout
- Support for GET, POST, PATCH, DELETE

### ✅ Request/Response Interceptors
- Add multiple request interceptors
- Add multiple response interceptors
- Add error interceptors
- Chain interceptors for complex logic

### ✅ Retry Logic
- Exponential backoff
- Configurable max retries
- Smart retry detection (5xx, 408, 429)
- Connection error handling

### ✅ API Error Handling
- Structured error objects
- Error code and message
- HTTP status codes
- Error details from response

### ✅ API Mocking
- Mock Service Worker (MSW) integration
- Mock handlers for all endpoints
- Development and testing support
- Easy enable/disable

## Usage

### Setup
```tsx
import { createApi } from '@stellar-escrow/api';

const api = createApi('http://localhost:3000');

// Add auth token
api.addAuthToken('your-token');

// Add error handler
api.addErrorHandler((error) => {
  console.error('API Error:', error);
});

// Add response logging
api.addResponseLogger();
```

`createApi()` normalizes bare origins to the versioned API base. For example,
`createApi('http://localhost:3000')` targets `http://localhost:3000/api`.

## Endpoint Matrix

| Area | Method | Route | Client method |
|------|--------|-------|---------------|
| Trades | `GET` | `/api/trades` | `trades.getTrades(limit, offset)` |
| Trades | `GET` | `/api/trades/:id` | `trades.getTrade(id)` |
| Trades | `POST` | `/api/trades` | `trades.createTrade(data)` |
| Trades | `PATCH` | `/api/trades/:id` | `trades.updateTrade(id, data)` |
| Trades | `DELETE` | `/api/trades/:id` | `trades.deleteTrade(id)` |
| Events | `GET` | `/api/events` | `events.getEvents(limit, tradeId?)` |
| Events | `GET` | `/api/events/trade/:tradeId` | `events.getEventsByTrade(tradeId)` |
| Events | `GET` | `/api/events/:id` | `events.getEvent(id)` |
| Blockchain | `POST` | `/api/blockchain/fund` | `blockchain.fundTrade(tradeId, amount)` |
| Blockchain | `POST` | `/api/blockchain/complete` | `blockchain.completeTrade(tradeId)` |
| Blockchain | `POST` | `/api/blockchain/resolve` | `blockchain.resolveDispute(tradeId, resolution)` |
| Blockchain | `GET` | `/api/blockchain/tx/:txHash` | `blockchain.getTransactionStatus(txHash)` |

### Trades API
```tsx
// Get all trades
const trades = await api.trades.getTrades(50, 0);

// Get single trade
const trade = await api.trades.getTrade('123');

// Create trade
const newTrade = await api.trades.createTrade({
  seller: 'G...',
  buyer: 'G...',
  amount: '100',
});

// Update trade
const updated = await api.trades.updateTrade('123', { status: 'funded' });

// Delete trade
await api.trades.deleteTrade('123');
```

### Events API
```tsx
// Get all events
const events = await api.events.getEvents(100);

// Get events for trade
const tradeEvents = await api.events.getEventsByTrade('123');

// Get single event
const event = await api.events.getEvent('456');
```

### Blockchain API
```tsx
// Fund trade
const fundTx = await api.blockchain.fundTrade('123', '100');

// Complete trade
const completeTx = await api.blockchain.completeTrade('123');

// Resolve dispute
const resolveTx = await api.blockchain.resolveDispute('123', 'release_to_buyer');

// Check transaction status
const status = await api.blockchain.getTransactionStatus(fundTx.txHash);
```

### Mocking
```tsx
import { server } from '@stellar-escrow/api/mocks';

// Enable mocking in tests
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// Use API with mocks
const trades = await api.trades.getTrades();
```

## Architecture

```
api/
├── src/
│   ├── client.ts        # Core API client
│   ├── resources.ts     # API resource classes
│   ├── mocks.ts         # MSW mock handlers
│   ├── models.ts        # TypeScript models
│   ├── types.ts         # Type definitions
│   ├── index.ts         # Main export
│   └── client.test.ts   # Tests
├── package.json
├── tsconfig.json
└── jest.config.js
```

## Interceptors

### Request Interceptor
```tsx
api.addRequestInterceptor((config) => {
  config.headers['X-Custom-Header'] = 'value';
  return config;
});
```

### Response Interceptor
```tsx
api.addResponseInterceptor((response) => {
  console.log('Response:', response.data);
  return response;
});
```

### Error Interceptor
```tsx
api.addErrorInterceptor(async (error) => {
  if (error.response?.status === 401) {
    // Handle unauthorized
  }
  throw error;
});
```

## Retry Configuration

```tsx
const api = createApi('http://localhost:3000');
// Default: 3 retries, 1000ms delay, 2x backoff
// Retries on: 5xx, 408, 429, connection errors
```

## Error Handling

```tsx
try {
  const trade = await api.trades.getTrade('123');
} catch (error) {
  console.error(error.code);      // 'ENOTFOUND', 'ECONNABORTED', etc.
  console.error(error.message);   // Error message
  console.error(error.status);    // HTTP status
  console.error(error.details);   // Response data
}
```

## Testing

```bash
npm test
npm run test:unit
npm run test:endpoints
npm run test:integration
npm run test:contract
npm run test:load
npm run test:stress
npm run test:benchmark
npm run test:scalability
npm run test:monitoring
npm run test:performance
npm run test:docs
```

Tests cover:
- Unit coverage for interceptors, config loading, retries, and endpoint mapping
- Integration coverage for mocked HTTP flows across trades, events, and blockchain routes
- Contract coverage for documented response shapes and endpoint registry drift
- Load coverage for concurrent request handling under representative read/write traffic
- Stress coverage for burst traffic and retry behaviour under transient failures
- Benchmark coverage for endpoint and workflow latency baselines
- Scalability coverage for throughput gains and latency regression checks across concurrency steps
- Monitoring coverage for threshold alerts, per-operation metrics, and failure visibility
- Documentation coverage to keep the README endpoint matrix aligned with the shipped client

## Performance Harness

The API package exports a lightweight performance harness for tests and local benchmarking:

```ts
import { PerformanceMonitor, executeScenario, evaluateThresholds } from '@stellar-escrow/api';

const monitor = new PerformanceMonitor();

const summary = await executeScenario(
  {
    name: 'trades-read-path',
    iterations: 20,
    concurrency: 5,
    thresholds: {
      maxP95LatencyMs: 200,
      maxErrorRate: 0,
    },
  },
  async (context) => {
    await context.measure('getTrades', () => api.trades.getTrades(10, context.iteration));
  },
  monitor
);

const alerts = evaluateThresholds(summary, { minThroughputPerSecond: 25 });
```

`PerformanceMonitor` records per-operation latency, success/error counts, and threshold alerts so load, stress, benchmark, scalability, and monitoring suites can share the same metrics model.
