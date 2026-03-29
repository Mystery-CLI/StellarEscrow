# Design Document: API Integration Layer

## Overview

This design introduces a thin centralized base query factory (`state/src/api/baseQuery.ts`) that
wraps RTK Query's `fetchBaseQuery` with timeout enforcement via `AbortController`, exponential-
backoff retry logic, normalized error shapes, and a `prepareHeaders` extension hook. The only
other change is a two-line edit to `state/src/api/escrowApi.ts` to consume the factory. Every
existing hook signature, `reducerPath`, tag type, and MSW handler remains untouched.

---

## Architecture

```
app/src/pages/
  └─ imports hooks from @stellar-escrow/state
        │
state/src/index.ts  (barrel — unchanged)
        │
state/src/api/escrowApi.ts  (modified: swap inline fetchBaseQuery → createBaseQuery)
        │
state/src/api/baseQuery.ts  (NEW)
        │
        ├─ fetchBaseQuery  (RTK Query built-in, used internally)
        ├─ AbortController (Web API / Node 18+)
        └─ fetch           (standard — intercepted by MSW in tests)
```

Data flow for a single request:

```
RTK Query calls baseQuery(args, api, extraOptions)
  → createBaseQuery resolves effective timeout & headers
  → AbortController.signal attached to fetch options
  → timeout timer set; on expiry → controller.abort()
  → fetchBaseQuery executes the fetch
  → on success  → return { data }
  → on HTTP err → return { error: NormalizedError }
  → on network  → return { error: NormalizedError }
  → on abort    → return { error: NormalizedError(TIMEOUT_ERROR) }
  → on retryable status → wait backoff delay → retry (up to maxRetries)
```

---

## Components and Interfaces

### `state/src/api/baseQuery.ts` — full module design

#### Type definitions

```typescript
// The shape of every error returned to RTK Query consumers.
// status is either an HTTP code (number) or a string sentinel.
export interface NormalizedError {
  status: number | 'FETCH_ERROR' | 'TIMEOUT_ERROR' | 'PARSING_ERROR';
  message: string;
  data?: unknown; // raw response body for HTTP errors, undefined otherwise
}

// Retry behaviour configuration.
export interface RetryConfig {
  maxRetries: number;       // default 3
  delayMs: number;          // base delay in ms, default 500
  backoffMultiplier: number;// multiplier per attempt, default 2
  // HTTP status codes that trigger a retry attempt.
  // Default: [408, 429, 500, 502, 503, 504]
  retryableStatuses: number[];
}

// Hook called before every request to mutate outgoing headers.
// Mirrors RTK Query's own prepareHeaders signature so callers can
// pass the same function to both fetchBaseQuery and createBaseQuery.
export type PrepareHeaders = (
  headers: Headers,
  api: { getState: () => unknown }
) => Headers | void | Promise<Headers | void>;

// Configuration accepted by createBaseQuery().
export interface BaseQueryConfig {
  baseUrl?: string;          // default: VITE_API_BASE_URL ?? '/api'
  timeoutMs?: number;        // default: 30_000
  prepareHeaders?: PrepareHeaders;
  retry?: Partial<RetryConfig>;
}
```

#### Factory function signature

```typescript
import { fetchBaseQuery, BaseQueryFn, FetchArgs, FetchBaseQueryError }
  from '@reduxjs/toolkit/query/react';

/**
 * Returns an RTK Query-compatible BaseQueryFn that adds timeout,
 * retry, and normalized error handling on top of fetchBaseQuery.
 */
export function createBaseQuery(config: BaseQueryConfig = {}): BaseQueryFn<
  string | FetchArgs,
  unknown,
  NormalizedError
>;
```

The returned function satisfies `BaseQueryFn<string | FetchArgs, unknown, NormalizedError>`,
which is the exact generic signature `createApi({ baseQuery })` expects.

#### Internal implementation sketch

```typescript
export function createBaseQuery(config: BaseQueryConfig = {}) {
  const baseUrl = config.baseUrl
    ?? (typeof import.meta !== 'undefined' ? import.meta.env?.VITE_API_BASE_URL : undefined)
    ?? '/api';

  const timeoutMs = config.timeoutMs ?? 30_000;

  const retry: RetryConfig = {
    maxRetries: 3,
    delayMs: 500,
    backoffMultiplier: 2,
    retryableStatuses: [408, 429, 500, 502, 503, 504],
    ...config.retry,
  };

  // Inner fetchBaseQuery instance — this is what actually calls fetch().
  const rawBaseQuery = fetchBaseQuery({
    baseUrl,
    prepareHeaders: config.prepareHeaders,
  });

  // The function returned to RTK Query.
  return async function baseQuery(
    args: string | FetchArgs,
    api: BaseQueryApi,
    extraOptions: Record<string, unknown>
  ): Promise<{ data: unknown } | { error: NormalizedError }> {
    return attemptRequest(args, api, extraOptions, 0);
  };

  async function attemptRequest(args, api, extraOptions, attempt) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);

    // Merge the abort signal into the request args.
    const argsWithSignal: FetchArgs =
      typeof args === 'string'
        ? { url: args, signal: controller.signal }
        : { ...args, signal: controller.signal };

    try {
      const result = await rawBaseQuery(argsWithSignal, api, extraOptions);
      clearTimeout(timer);

      if (result.error) {
        return handleError(result.error, args, api, extraOptions, attempt);
      }
      return result; // { data }
    } catch (thrown) {
      clearTimeout(timer);
      // AbortController fires a DOMException with name 'AbortError'.
      if (thrown instanceof DOMException && thrown.name === 'AbortError') {
        return {
          error: {
            status: 'TIMEOUT_ERROR',
            message: `Request timed out after ${timeoutMs}ms`,
          },
        };
      }
      return {
        error: {
          status: 'FETCH_ERROR',
          message: thrown instanceof Error ? thrown.message : 'Network error',
        },
      };
    }
  }

  async function handleError(rtkError, args, api, extraOptions, attempt) {
    const normalized = normalizeError(rtkError);

    // Retry on retryable HTTP status codes.
    if (
      attempt < retry.maxRetries &&
      typeof normalized.status === 'number' &&
      retry.retryableStatuses.includes(normalized.status)
    ) {
      const delay = retry.delayMs * Math.pow(retry.backoffMultiplier, attempt);
      await sleep(delay);
      return attemptRequest(args, api, extraOptions, attempt + 1);
    }

    return { error: normalized };
  }
}

function normalizeError(rtkError: FetchBaseQueryError): NormalizedError {
  if (typeof rtkError.status === 'number') {
    // HTTP error — rtkError.data is the parsed response body.
    return {
      status: rtkError.status,
      message: httpMessage(rtkError.status, rtkError.data),
      data: rtkError.data,
    };
  }
  // RTK sentinel strings: 'FETCH_ERROR', 'PARSING_ERROR', etc.
  return {
    status: rtkError.status as NormalizedError['status'],
    message: (rtkError as any).error ?? 'Unknown error',
  };
}

function httpMessage(status: number, data: unknown): string {
  if (data && typeof data === 'object' && 'message' in data) {
    return String((data as any).message);
  }
  return `HTTP ${status}`;
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
```

#### Timeout enforcement detail

- An `AbortController` is created per request attempt (not shared across retries).
- `setTimeout(controller.abort, timeoutMs)` is set before `rawBaseQuery` is called.
- `clearTimeout` is called in both the success and error paths to avoid leaks.
- When `controller.abort()` fires, `fetch` rejects with a `DOMException { name: 'AbortError' }`.
- The catch block checks `thrown.name === 'AbortError'` and returns `{ error: { status: 'TIMEOUT_ERROR', ... } }`.
- Per-request timeout: callers can pass `extraOptions.timeoutMs` — the factory can read this
  from `extraOptions` and override `timeoutMs` for that call only.

#### Retry logic detail

| Parameter | Default | Notes |
|---|---|---|
| `maxRetries` | 3 | Total extra attempts after the first failure |
| `delayMs` | 500 | Base delay before first retry |
| `backoffMultiplier` | 2 | Delay doubles each attempt: 500ms, 1000ms, 2000ms |
| `retryableStatuses` | [408, 429, 500, 502, 503, 504] | Only HTTP errors retry; FETCH_ERROR and TIMEOUT_ERROR do not |

Retry does NOT apply to:
- `FETCH_ERROR` (network down — retrying immediately is unlikely to help)
- `TIMEOUT_ERROR` (the request already consumed the full timeout budget)
- 4xx errors other than 408/429 (client errors are not transient)

#### prepareHeaders hook

`createBaseQuery` passes `config.prepareHeaders` directly to `fetchBaseQuery`'s own
`prepareHeaders` option. This means:
- RTK Query calls it with `(headers: Headers, { getState })` before every request.
- The hook can mutate the `Headers` object in place or return a new one.
- Future auth token injection: `headers.set('Authorization', 'Bearer ' + getToken(getState()))`.
- No changes to `createBaseQuery` are needed when auth is added — callers just pass the hook.

#### Exports

```typescript
// Named exports from state/src/api/baseQuery.ts
export { createBaseQuery };
export type { BaseQueryConfig, NormalizedError, RetryConfig, PrepareHeaders };
```

---

## Data Models

### NormalizedError (canonical shape)

```typescript
// HTTP error (4xx / 5xx)
{ status: 404, message: 'HTTP 404', data: { error: 'Not found' } }

// Network failure
{ status: 'FETCH_ERROR', message: 'Failed to fetch' }

// Timeout
{ status: 'TIMEOUT_ERROR', message: 'Request timed out after 30000ms' }
```

RTK Query consumers access this via the hook's `error` field:

```typescript
const { data, error, isLoading } = useGetTradesQuery({});
if (error) {
  // error is typed as NormalizedError
  console.log(error.status, error.message);
}
```

### BaseQueryConfig (instantiation shape)

```typescript
// Minimal — uses all defaults
createBaseQuery()

// Full override example for a hypothetical second slice
createBaseQuery({
  baseUrl: 'https://other-service.example.com',
  timeoutMs: 10_000,
  retry: { maxRetries: 1 },
  prepareHeaders: (headers, { getState }) => {
    headers.set('Authorization', 'Bearer ' + selectToken(getState()));
    return headers;
  },
})
```

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a
system — essentially, a formal statement about what the system should do. Properties serve as the
bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: BaseQueryFn return shape

*For any* valid request args (string URL or FetchArgs object), the function returned by
`createBaseQuery` must return either `{ data: unknown }` on success or `{ error: NormalizedError }`
on failure — never both, never neither.

**Validates: Requirements 1.3, 4.2**

---

### Property 2: Base URL resolution

*For any* combination of `config.baseUrl`, `VITE_API_BASE_URL` env var, and absence of both,
the resolved base URL used for requests must be: `config.baseUrl` if provided, else
`VITE_API_BASE_URL` if set, else `/api`.

**Validates: Requirements 1.2, 5.4**

---

### Property 3: prepareHeaders is called on every request

*For any* `prepareHeaders` function passed to `createBaseQuery`, that function must be invoked
exactly once per request attempt, receiving a `Headers` instance and the RTK Query `api` object.

**Validates: Requirements 1.5**

---

### Property 4: Content-Type header on body requests

*For any* request that includes a non-empty body, the outgoing `Content-Type` header must be
`application/json`. Requests without a body must not have this header injected.

**Validates: Requirements 2.2**

---

### Property 5: Timeout resolution — default vs per-request override

*For any* `BaseQueryConfig` with optional `timeoutMs` and any per-request `extraOptions.timeoutMs`,
the effective timeout applied to the `AbortController` must be the per-request value when
provided, otherwise the config-level value, otherwise 30 000 ms.

**Validates: Requirements 2.1, 2.3**

---

### Property 6: HTTP error normalization shape

*For any* HTTP response with a 4xx or 5xx status code, `createBaseQuery` must return
`{ error: { status: <http-code>, message: <string>, data: <response-body> } }` where `status`
is the numeric HTTP code, `message` is a non-empty string, and `data` is the parsed response body.

**Validates: Requirements 3.1**

---

## Error Handling

| Scenario | Detected by | Returned shape |
|---|---|---|
| HTTP 4xx/5xx | `result.error.status` is a number | `{ error: { status: N, message, data } }` |
| Network failure | `fetch` throws (non-abort) | `{ error: { status: 'FETCH_ERROR', message } }` |
| Timeout | `DOMException { name: 'AbortError' }` | `{ error: { status: 'TIMEOUT_ERROR', message } }` |
| RTK parsing error | `result.error.status === 'PARSING_ERROR'` | `{ error: { status: 'PARSING_ERROR', message } }` |

All error paths return `{ error: NormalizedError }`. No path returns `undefined` or throws.
RTK Query propagates `{ error }` to the hook's `error` field automatically — UI components need
no `try/catch`.

---

## Testing Strategy

### Dual approach

Both unit tests and property-based tests are required. They are complementary:
- Unit tests catch concrete bugs at specific inputs and integration points.
- Property tests verify universal correctness across the full input space.

### Unit tests (specific examples and edge cases)

Located in `state/src/api/baseQuery.test.ts`:

- Default timeout fires after 30 000 ms (mock timers + fake fetch that never resolves)
- Network failure returns `{ status: 'FETCH_ERROR' }`
- Timeout returns `{ status: 'TIMEOUT_ERROR' }`
- RTK Query `error` field is populated when `baseQuery` returns `{ error }`
- `escrowApi` exports all 6 hooks with unchanged names (TypeScript compilation + import check)
- `escrowApi.reducerPath === 'escrowApi'` and `tagTypes` unchanged (snapshot)
- MSW integration: existing handlers intercept requests from `createBaseQuery` without modification

### Property-based tests

Use [fast-check](https://github.com/dubzzz/fast-check) (already compatible with Jest/Vitest).
Configure each test with minimum 100 runs.

```typescript
// Tag format: Feature: api-integration-layer, Property N: <property text>
```

**Property 1 — BaseQueryFn return shape**
```
// Feature: api-integration-layer, Property 1: BaseQueryFn return shape
fc.assert(fc.asyncProperty(
  fc.oneof(fc.string(), fc.record({ url: fc.string() })),
  async (args) => {
    const result = await baseQuery(args, mockApi, {});
    return ('data' in result) !== ('error' in result); // exactly one
  }
), { numRuns: 100 });
```

**Property 2 — Base URL resolution**
```
// Feature: api-integration-layer, Property 2: Base URL resolution
fc.assert(fc.property(
  fc.option(fc.webUrl()), fc.option(fc.webUrl()),
  (configUrl, envUrl) => {
    // set/unset VITE_API_BASE_URL, call createBaseQuery({ baseUrl: configUrl })
    // verify resolved URL matches priority order
  }
), { numRuns: 100 });
```

**Property 3 — prepareHeaders called on every request**
```
// Feature: api-integration-layer, Property 3: prepareHeaders is called on every request
fc.assert(fc.asyncProperty(
  fc.array(fc.string(), { minLength: 1, maxLength: 5 }),
  async (urls) => {
    let callCount = 0;
    const bq = createBaseQuery({ prepareHeaders: (h) => { callCount++; return h; } });
    for (const url of urls) await bq(url, mockApi, {});
    return callCount === urls.length;
  }
), { numRuns: 100 });
```

**Property 4 — Content-Type on body requests**
```
// Feature: api-integration-layer, Property 4: Content-Type header on body requests
fc.assert(fc.asyncProperty(
  fc.record({ url: fc.string(), body: fc.object() }),
  async (args) => {
    // intercept headers via prepareHeaders spy
    // verify Content-Type: application/json is present
  }
), { numRuns: 100 });
```

**Property 5 — Timeout resolution**
```
// Feature: api-integration-layer, Property 5: Timeout resolution
fc.assert(fc.property(
  fc.option(fc.integer({ min: 1000, max: 60000 })),
  fc.option(fc.integer({ min: 1000, max: 60000 })),
  (configTimeout, perRequestTimeout) => {
    const expected = perRequestTimeout ?? configTimeout ?? 30_000;
    // verify AbortController fires at `expected` ms
  }
), { numRuns: 100 });
```

**Property 6 — HTTP error normalization shape**
```
// Feature: api-integration-layer, Property 6: HTTP error normalization shape
fc.assert(fc.asyncProperty(
  fc.integer({ min: 400, max: 599 }),
  fc.anything(),
  async (statusCode, responseBody) => {
    // mock fetch to return statusCode with responseBody
    const result = await baseQuery('/test', mockApi, {});
    return (
      'error' in result &&
      result.error.status === statusCode &&
      typeof result.error.message === 'string' &&
      result.error.message.length > 0
    );
  }
), { numRuns: 100 });
```

### MSW compatibility note

The existing MSW handlers (`api/src/mocks/handlers/trades.ts`, `events.ts`) intercept standard
`fetch` calls at the network level. Because `createBaseQuery` delegates to RTK Query's
`fetchBaseQuery` — which uses the global `fetch` — MSW intercepts requests transparently.
No handler changes are required. This is in contrast to the backend `api/src/client.ts` which
uses axios; axios requires a separate MSW adapter or a custom adapter to be intercepted.

---

## Migration Path

### Step 1 — Create `state/src/api/baseQuery.ts`

Create the new file with `createBaseQuery` and all type exports. At this point `escrowApi.ts`
still uses its inline `fetchBaseQuery` call and remains fully functional. The test suite passes
unchanged.

### Step 2 — Update `state/src/api/escrowApi.ts`

Two changes only:

```typescript
// Before
import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';

export const escrowApi = createApi({
  reducerPath: 'escrowApi',
  baseQuery: fetchBaseQuery({ baseUrl: '/api' }),
  // ...
});
```

```typescript
// After
import { createApi } from '@reduxjs/toolkit/query/react';
import { createBaseQuery } from './baseQuery';

export const escrowApi = createApi({
  reducerPath: 'escrowApi',          // unchanged
  baseQuery: createBaseQuery(),       // uses VITE_API_BASE_URL ?? '/api', 30s timeout, 3 retries
  tagTypes: ['Trade', 'Event'],       // unchanged
  endpoints: (builder) => ({ /* ... all 6 endpoints unchanged ... */ }),
});

// All 6 hook exports unchanged:
export const {
  useGetTradesQuery,
  useGetTradeQuery,
  useCreateTradeMutation,
  useUpdateTradeMutation,
  useGetEventsQuery,
  useGetEventsByTradeQuery,
} = escrowApi;
```

`reducerPath`, `tagTypes`, and every endpoint definition are byte-for-byte identical to today.

### Step 3 — Run test suite

```bash
# from state/ package
npx jest --run
```

All existing tests should pass. New tests in `baseQuery.test.ts` are added in this step.

---

## MSW Compatibility Note

`createBaseQuery` uses `fetchBaseQuery` internally, which calls the global `fetch`. MSW's
`setupServer` (Node) and `setupWorker` (browser) both intercept at the `fetch` level. This means:

- Every request from `createBaseQuery` is intercepted by the existing handlers.
- No handler changes are required.
- No MSW adapter is needed (unlike axios, which requires `msw-axios-adapter`).
- The `AbortController` signal is passed through to `fetch`. MSW v1 (`rest` handlers) ignores
  the signal on the mock side, so abort/timeout tests must use real timers or mock `fetch`
  directly rather than relying on MSW to simulate slow responses.

---

## Risks Carried Forward

### 1. AbortController + MSW interaction

MSW v1 does not honour the `AbortController` signal on intercepted requests — the mock response
is returned synchronously before the abort fires. This means timeout tests cannot use MSW to
simulate a slow server; they must either mock `fetch` directly or use `jest.useFakeTimers()` with
a fetch that never resolves. This is a test-authoring concern only and does not affect production
behaviour.

### 2. VITE_API_BASE_URL in Jest/Node environments

`import.meta.env` is a Vite-specific construct. In Jest (which runs in Node via ts-jest or
babel-jest), `import.meta` may be undefined or empty. The factory guards against this:

```typescript
const baseUrl = config.baseUrl
  ?? (typeof import.meta !== 'undefined' ? import.meta.env?.VITE_API_BASE_URL : undefined)
  ?? '/api';
```

Tests that need a specific base URL should pass `config.baseUrl` explicitly rather than relying
on the env var. The `/api` fallback ensures tests that do not set either still resolve to a
consistent value that MSW handlers are registered against.
