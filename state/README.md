# Redux State Management

Optimized Redux store with RTK Query, state normalization, persistence, and debugging tools.

## Architecture

### Store Structure
```
state/
├── src/
│   ├── slices/              # Redux slices (trades, events, ui)
│   ├── api/                 # RTK Query API definitions
│   ├── selectors.ts         # Memoized selectors
│   ├── store.ts             # Store configuration
│   ├── devtools.ts          # Debugging utilities
│   ├── types.ts             # TypeScript types
│   └── index.ts             # Main export
├── package.json
├── tsconfig.json
└── jest.config.js
```

## Features

### ✅ Restructured Redux Store
- **Normalized State**: Trades and events stored by ID for efficient updates
- **Slices**: Separate slices for trades, events, and UI state
- **Type Safety**: Full TypeScript support with strict types

### ✅ RTK Query for API Calls
- **Automatic Caching**: Built-in cache management
- **Tag-Based Invalidation**: Smart cache invalidation
- **Hooks**: `useGetTradesQuery`, `useCreateTradeMutation`, etc.
- **Error Handling**: Built-in error states

### ✅ State Normalization
- **Normalized Structure**: `{ byId: {}, allIds: [] }`
- **Efficient Updates**: O(1) lookups and updates
- **Selectors**: Memoized selectors for derived state

### ✅ State Persistence
- **Redux Persist**: Automatic state persistence to localStorage
- **Selective Persistence**: Only persist trades and UI state
- **Rehydration**: Automatic state restoration on app load

### ✅ State Debugging Tools
- **Redux Logger**: Middleware for action logging
- **Redux DevTools**: Browser extension support
- **State Snapshots**: `getStateSnapshot()` function
- **State Tree Logging**: `logStateTree()` function

## Usage

### Setup Store
```tsx
import { store, persistor } from '@stellar-escrow/state';
import { PersistGate } from 'redux-persist/integration/react';
import { Provider } from 'react-redux';

function App() {
  return (
    <Provider store={store}>
      <PersistGate loading={null} persistor={persistor}>
        <YourApp />
      </PersistGate>
    </Provider>
  );
}
```

### Use Hooks
```tsx
import { useGetTradesQuery, useCreateTradeMutation } from '@stellar-escrow/state';

function Trades() {
  const { data: trades, isLoading } = useGetTradesQuery({ limit: 50 });
  const [createTrade] = useCreateTradeMutation();

  return (
    <div>
      {isLoading ? 'Loading...' : trades?.map(t => <div key={t.id}>{t.id}</div>)}
    </div>
  );
}
```

### Use Selectors
```tsx
import { useSelector } from 'react-redux';
import { selectAllTrades, selectTradesByStatus } from '@stellar-escrow/state';

function TradeList() {
  const trades = useSelector(selectAllTrades);
  const completed = useSelector(state => selectTradesByStatus(state, 'completed'));

  return <div>{trades.length} trades</div>;
}
```

### Debugging
```tsx
import { logStateTree, getStateSnapshot } from '@stellar-escrow/state';

// Log entire state tree
logStateTree();

// Get state snapshot
const state = getStateSnapshot();
console.log(state);
```

## State Shape

```typescript
{
  trades: {
    byId: {
      '1': { id: '1', seller: '...', buyer: '...', ... },
      '2': { id: '2', ... }
    },
    allIds: ['1', '2'],
    loading: false,
    error: null
  },
  events: {
    byId: { ... },
    allIds: [],
    loading: false,
    error: null
  },
  ui: {
    selectedTradeId: null,
    filters: {},
    pagination: { page: 1, pageSize: 50 }
  },
  escrowApi: { ... }  // RTK Query cache
}
```

## Selectors

### Trades
- `selectAllTrades(state)` - All trades as array
- `selectTradeById(state, id)` - Single trade by ID
- `selectTradesByStatus(state, status)` - Trades filtered by status
- `selectTradesLoading(state)` - Loading state
- `selectTradesError(state)` - Error state

### Events
- `selectAllEvents(state)` - All events as array
- `selectEventsByTradeId(state, tradeId)` - Events for trade
- `selectEventsLoading(state)` - Loading state
- `selectEventsError(state)` - Error state

### UI
- `selectSelectedTradeId(state)` - Selected trade ID
- `selectFilters(state)` - Current filters
- `selectPagination(state)` - Pagination state

## API Endpoints

### Trades
- `GET /api/trades` - List trades
- `GET /api/trades/:id` - Get trade
- `POST /api/trades` - Create trade
- `PATCH /api/trades/:id` - Update trade

### Events
- `GET /api/events` - List events
- `GET /api/events/trade/:tradeId` - Events for trade

## Middleware

- **RTK Query**: Automatic API caching and synchronization
- **Redux Logger**: Action and state logging
- **Redux Persist**: State persistence to localStorage
- **Redux DevTools**: Browser extension integration

## Testing

```bash
npm test
```

Tests cover:
- Store initialization
- Trade actions (add, update, set)
- Event actions (add, set)
- State normalization
- Selector functionality

## Performance

- **Normalized State**: O(1) lookups and updates
- **Memoized Selectors**: Prevent unnecessary re-renders
- **RTK Query Caching**: Automatic cache management
- **Selective Persistence**: Only persist necessary state

## Next Steps

1. Integrate with React components
2. Add more API endpoints as needed
3. Implement optimistic updates
4. Add state validation
5. Set up time-travel debugging
