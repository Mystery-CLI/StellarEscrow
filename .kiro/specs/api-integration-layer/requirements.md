# Requirements Document

## Introduction

The frontend currently makes HTTP calls through RTK Query's `fetchBaseQuery` configured directly
inside `state/src/api/escrowApi.ts`. This works, but the raw `fetchBaseQuery` is not wrapped in
any shared client abstraction, so concerns like auth headers, base-URL configuration, request
timeouts, and error normalisation are either absent or duplicated if a second API slice is ever
added. This feature introduces a thin, centralized frontend API client — living entirely inside
`state/src/api/` — that `escrowApi` (and any future RTK Query slices) delegate to, without
touching the backend `api/src/client.ts` or the Soroban contract layer.

---

## Glossary

- **Frontend_API_Client**: A lightweight wrapper around RTK Query's `fetchBaseQuery` that lives in
  `state/src/api/baseQuery.ts` and provides shared configuration (base URL, headers, timeout,
  error normalisation) to all RTK Query slices.
- **escrowApi**: The existing RTK Query API slice defined in `state/src/api/escrowApi.ts` that
  exposes trade and event endpoints to UI pages.
- **RTK_Query**: Redux Toolkit Query — the data-fetching and caching layer already used by the
  frontend (`@reduxjs/toolkit/query/react`).
- **MSW**: Mock Service Worker — the HTTP mocking library already installed and used in
  `api/src/mocks/` for tests.
- **Base_URL**: The root URL prefix (`/api`) prepended to every endpoint path.
- **Normalized_Error**: A typed error object with a consistent shape (`{ status, message, data }`)
  returned by the Frontend_API_Client on any failed request.
- **Tag**: An RTK Query cache tag (e.g. `'Trade'`, `'Event'`) used to drive cache invalidation.
- **Backend_ApiClient**: The axios-based `ApiClient` class in `api/src/client.ts`. Out of scope —
  must not be modified.

---

## Requirements

### Requirement 1: Centralized Base Query

**User Story:** As a frontend developer, I want a single place to configure HTTP settings for all
frontend API calls, so that changes to base URL, headers, or timeout only need to be made once.

#### Acceptance Criteria

1. THE Frontend_API_Client SHALL be defined in `state/src/api/baseQuery.ts`.
2. THE Frontend_API_Client SHALL read the Base_URL from an environment variable
   (`VITE_API_BASE_URL`) and fall back to `/api` when the variable is absent.
3. THE Frontend_API_Client SHALL expose a `baseQuery` function compatible with RTK Query's
   `BaseQueryFn` interface so it can be passed directly to `createApi`.
4. WHEN `escrowApi` is initialised, THE escrowApi SHALL use the Frontend_API_Client's `baseQuery`
   instead of an inline `fetchBaseQuery` call.
5. THE Frontend_API_Client SHALL accept a `prepareHeaders` hook so that future auth token
   injection can be added without modifying individual endpoint definitions.

---

### Requirement 2: Request Configuration

**User Story:** As a frontend developer, I want consistent request defaults applied to every API
call, so that I do not have to repeat timeout or content-type settings per endpoint.

#### Acceptance Criteria

1. THE Frontend_API_Client SHALL set a default request timeout of 30 000 ms for every outgoing
   request.
2. THE Frontend_API_Client SHALL set `Content-Type: application/json` on every request that
   includes a body.
3. WHERE a custom timeout is provided per-request, THE Frontend_API_Client SHALL use the
   per-request value instead of the default.

---

### Requirement 3: Error Normalisation

**User Story:** As a frontend developer, I want API errors to have a predictable shape, so that
UI components can display error messages without inspecting raw HTTP responses.

#### Acceptance Criteria

1. WHEN a request returns an HTTP error status (4xx or 5xx), THE Frontend_API_Client SHALL return
   a Normalized_Error containing `status` (HTTP status code), `message` (human-readable string),
   and `data` (raw response body, if any).
2. WHEN a network failure occurs (no response received), THE Frontend_API_Client SHALL return a
   Normalized_Error with `status: 'FETCH_ERROR'` and a descriptive `message`.
3. WHEN a request times out, THE Frontend_API_Client SHALL return a Normalized_Error with
   `status: 'TIMEOUT_ERROR'` and a descriptive `message`.
4. THE Frontend_API_Client SHALL NOT swallow errors silently; every error path SHALL produce a
   Normalized_Error.
5. IF a UI page receives a Normalized_Error, THEN THE escrowApi SHALL surface the error through
   RTK Query's standard `error` field so the page can render an error state without additional
   try/catch blocks.

---

### Requirement 4: Compatibility with RTK Query Caching and Invalidation

**User Story:** As a frontend developer, I want the new base query to preserve all existing RTK
Query caching and tag-invalidation behaviour, so that no UI regressions are introduced.

#### Acceptance Criteria

1. WHEN `escrowApi` uses the Frontend_API_Client, THE escrowApi SHALL continue to provide and
   invalidate Tags (`'Trade'`, `'Event'`) exactly as defined in the current endpoint
   configuration.
2. THE Frontend_API_Client SHALL return responses in the `{ data }` / `{ error }` shape required
   by RTK Query's `BaseQueryFn` contract so that caching, polling, and invalidation work without
   modification.
3. WHEN a mutation invalidates a Tag, THE RTK_Query cache SHALL refetch the affected queries
   exactly as it does today.
4. THE Frontend_API_Client SHALL NOT introduce additional Redux state outside of the existing
   `escrowApi.reducerPath`.

---

### Requirement 5: Compatibility with MSW Tests

**User Story:** As a frontend developer, I want existing MSW-based tests to continue passing
after the base query is introduced, so that the test suite remains green without rewriting mocks.

#### Acceptance Criteria

1. THE Frontend_API_Client SHALL issue standard `fetch` requests so that MSW can intercept them
   using the existing handlers in `api/src/mocks/handlers/`.
2. WHEN MSW intercepts a request, THE Frontend_API_Client SHALL process the mocked response
   identically to a real server response.
3. THE Frontend_API_Client SHALL NOT require any changes to existing MSW handler definitions in
   `api/src/mocks/handlers/trades.ts` or `api/src/mocks/handlers/events.ts`.
4. WHERE a test overrides the Base_URL, THE Frontend_API_Client SHALL use the overridden value so
   that tests can point at a local MSW server without environment variable changes.

---

### Requirement 6: Migration of escrowApi

**User Story:** As a frontend developer, I want `escrowApi` to be migrated to use the
Frontend_API_Client with no change to the public hook API, so that all UI pages continue to work
without modification.

#### Acceptance Criteria

1. WHEN the migration is complete, THE escrowApi SHALL export the same hooks
   (`useGetTradesQuery`, `useGetTradeQuery`, `useCreateTradeMutation`, `useUpdateTradeMutation`,
   `useGetEventsQuery`, `useGetEventsByTradeQuery`) with identical signatures.
2. THE escrowApi SHALL NOT change its `reducerPath` (`'escrowApi'`) or its Tag definitions.
3. WHEN a UI page imports a hook from `@stellar-escrow/state`, THE page SHALL require no code
   changes as a result of this migration.
4. THE `state/src/index.ts` barrel export SHALL continue to re-export all hooks and the
   `escrowApi` object without modification to the import paths consumed by `app/src/pages/`.

---

### Requirement 7: File-Level Impact Boundary

**User Story:** As a frontend developer, I want the change set to be small and contained, so that
code review is straightforward and the risk of unintended side-effects is low.

#### Acceptance Criteria

1. THE implementation SHALL create exactly one new file: `state/src/api/baseQuery.ts`.
2. THE implementation SHALL modify `state/src/api/escrowApi.ts` only to replace the inline
   `fetchBaseQuery` call with the Frontend_API_Client's `baseQuery`.
3. THE implementation SHALL NOT modify any file outside of `state/src/api/`.
4. THE implementation SHALL NOT modify `api/src/client.ts`, `client/src/lib/contract.ts`, or any
   file in `app/src/pages/`.

---

### Requirement 8: Extensibility for Future Slices

**User Story:** As a frontend developer, I want the Frontend_API_Client to be reusable by future
RTK Query slices, so that new API domains do not need to re-implement shared HTTP concerns.

#### Acceptance Criteria

1. THE Frontend_API_Client SHALL be exported from `state/src/api/baseQuery.ts` as a named export
   so that future slices can import it directly.
2. THE Frontend_API_Client SHALL accept optional configuration overrides (e.g. a different
   Base_URL or `prepareHeaders` function) so that a future slice targeting a different service
   can reuse the same factory without forking the implementation.
3. THE `state/src/index.ts` barrel SHALL NOT need to be updated when a new slice is added that
   imports from `state/src/api/baseQuery.ts` directly.

---

### Requirement 9: Migration Plan

**User Story:** As a frontend developer, I want a clear, low-risk migration sequence, so that the
codebase is never left in a broken intermediate state.

#### Acceptance Criteria

1. THE migration SHALL be executed in the following order:
   a. Create `state/src/api/baseQuery.ts` with the Frontend_API_Client implementation.
   b. Update `state/src/api/escrowApi.ts` to import and use the Frontend_API_Client.
   c. Run the existing test suite to verify no regressions.
2. WHEN step (a) is complete and step (b) has not yet started, THE existing `escrowApi` SHALL
   remain fully functional using the original inline `fetchBaseQuery`.
3. THE migration SHALL NOT require a database migration, environment variable change in
   production, or deployment of the backend service.

---

## Risks and Open Questions

1. **Timeout enforcement via `fetchBaseQuery`**: RTK Query's `fetchBaseQuery` does not natively
   support per-request timeouts. The Frontend_API_Client may need to wrap requests with
   `AbortController` to honour the 30 000 ms default. This should be validated during design.

2. **Auth header injection**: No authentication mechanism exists today. The `prepareHeaders` hook
   is included as a forward-looking extension point. If auth is added later, the hook signature
   must be agreed before the Frontend_API_Client is finalised.

3. **MSW version compatibility**: The existing handlers use `msw` v1 (`rest` import). If the
   project upgrades to MSW v2 (`http` import), handler syntax changes but the Frontend_API_Client
   itself is unaffected since it uses standard `fetch`.

4. **Environment variable availability**: `VITE_API_BASE_URL` is a Vite-specific env var prefix.
   If the `state` package is ever consumed outside a Vite build (e.g. in Jest/Node tests), the
   variable may be undefined. The `/api` fallback mitigates this, but test environments should
   confirm the base URL resolves correctly against the MSW server.

5. **Scope creep**: This spec intentionally excludes retry logic, request deduplication, and
   response caching beyond what RTK Query already provides. These concerns should be tracked as
   separate issues if needed.
