# Advanced Filters Utility

A high-performance utility function for filtering and sorting trade data in StellarEscrow. Efficiently handles multi-criteria filtering, complex sorting, and large datasets.

## Location

- **File**: `components/src/utils/filters.ts`
- **Tests**: `components/src/utils/filters.test.ts`
- **Exports**: Via `components/src/utils/index.ts` and `components/src/index.ts`

## Features

✅ **Multi-Criteria Filtering**
- String filters with case-insensitive substring matching (status, seller, buyer, arbitrator)
- Range filters for numeric values (minAmount, maxAmount)
- Range filters for dates (startDate, endDate)
- Boolean filters (isVerified, isDisputed)
- Support for custom filter fields

✅ **Complex Sorting**
- Support for multiple sort criteria
- Primary and secondary sorting with independent directions
- Handles strings (case-insensitive), numbers, and dates intelligently
- Null/undefined value handling

✅ **Performance Optimized**
- Single-pass filtering algorithm
- Efficient array operations (no unnecessary iterations)
- Lazy evaluation where possible
- Handles 10,000+ items efficiently (<500ms)
- Non-mutating (preserves original array)

## API Reference

### `applyAdvancedFilters(data, filters?, sortConfig?)`

Main function for filtering and sorting data.

**Parameters:**

- `data: T[]` - Array of trade objects to filter and sort
- `filters?: Partial<FilterCriteria>` - Filter criteria (optional)
- `sortConfig?: SortConfig[]` - Array of sort configurations (optional)

**Returns:** `T[]` - Filtered and sorted array

**Generic Type:** `T extends Record<string, any>`

### `FilterCriteria` Interface

```typescript
interface FilterCriteria {
  // String filters (substring match, case-insensitive)
  status?: string;           // e.g., 'completed', 'disputed'
  seller?: string;           // Substring match on seller name
  buyer?: string;            // Substring match on buyer name
  arbitrator?: string;       // Substring match on arbitrator name
  tradeId?: string;          // Exact match on trade ID

  // Range filters (numeric)
  minAmount?: number | string;
  maxAmount?: number | string;

  // Range filters (date)
  startDate?: string | number;  // ISO string or timestamp
  endDate?: string | number;    // ISO string or timestamp

  // Boolean filters
  isVerified?: boolean;
  isDisputed?: boolean;

  // Custom filters
  [key: string]: any;
}
```

### `SortConfig` Interface

```typescript
interface SortConfig {
  key: string;                    // Field name to sort by
  direction: 'asc' | 'desc';      // Sort direction
}
```

### `validateFilters(filters)`

Optional validation utility to check filter correctness.

**Parameters:**

- `filters: Partial<FilterCriteria>` - Filters to validate

**Returns:** `boolean` - True if valid, false otherwise

**Validates:**
- minAmount ≤ maxAmount
- startDate ≤ endDate

## Usage Examples

### Basic Filtering

```typescript
import { applyAdvancedFilters, FilterCriteria } from '@components';

const trades = [/* trade data */];

// Filter by status
const filters: FilterCriteria = { status: 'completed' };
const completed = applyAdvancedFilters(trades, filters);

// Filter by seller (substring match)
const sellerFilters: FilterCriteria = { seller: 'alice' };
const aliceTrades = applyAdvancedFilters(trades, sellerFilters);
```

### Range Filtering

```typescript
// Filter by amount range
const amountFilters: FilterCriteria = {
  minAmount: 100,
  maxAmount: 5000
};
const inRange = applyAdvancedFilters(trades, amountFilters);

// Filter by date range
const dateFilters: FilterCriteria = {
  startDate: '2024-01-01',
  endDate: '2024-12-31'
};
const thisYear = applyAdvancedFilters(trades, dateFilters);
```

### Boolean Filtering

```typescript
// Filter verified trades that are not disputed
const boolFilters: FilterCriteria = {
  isVerified: true,
  isDisputed: false
};
const safeTrades = applyAdvancedFilters(trades, boolFilters);
```

### Multi-Criteria Filtering

```typescript
// Combine multiple filter types (AND logic)
const complexFilters: FilterCriteria = {
  status: 'completed',
  seller: 'alice',
  minAmount: 500,
  maxAmount: 5000,
  isVerified: true,
  startDate: '2024-01-01'
};
const results = applyAdvancedFilters(trades, complexFilters);
```

### Sorting

```typescript
import { SortConfig } from '@components';

// Single sort criterion
const sortAsc: SortConfig[] = [{ key: 'amount', direction: 'asc' }];
const ascending = applyAdvancedFilters(trades, {}, sortAsc);

// Multiple sort criteria (chained)
const multiSort: SortConfig[] = [
  { key: 'isVerified', direction: 'desc' },      // Primary: verified first
  { key: 'amount', direction: 'desc' }           // Secondary: highest amount
];
const prioritized = applyAdvancedFilters(trades, {}, multiSort);

// Sort by date descending
const byDate: SortConfig[] = [{ key: 'timestamp', direction: 'desc' }];
const recent = applyAdvancedFilters(trades, {}, byDate);
```

### Combined Filtering and Sorting

```typescript
// Filter AND then sort
const filters: FilterCriteria = {
  status: 'completed',
  minAmount: 1000
};

const sortConfig: SortConfig[] = [
  { key: 'timestamp', direction: 'desc' },
  { key: 'amount', direction: 'asc' }
];

const results = applyAdvancedFilters(trades, filters, sortConfig);
```

### Validation

```typescript
// Validate filters before applying
const filters: FilterCriteria = {
  minAmount: 5000,
  maxAmount: 1000  // Invalid! maxAmount < minAmount
};

if (validateFilters(filters)) {
  applyAdvancedFilters(trades, filters);
} else {
  console.warn('Invalid filters provided');
}
```

## Implementation Details

### Filter Logic (AND-based)

All filter criteria are combined with **AND logic**:
- Trade must match status AND seller AND amount range AND date range, etc.
- If any criterion fails, the trade is excluded

### Sorting Algorithm

- Sorts are applied in reverse order for proper precedence
- Primary sort (first in array) takes precedence
- Secondary sorts apply only within same primary sort values
- String comparisons are case-insensitive and locale-aware
- Null/undefined values sort last

### Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Single filter | O(n) | Single-pass through array |
| Multiple filters | O(n) | Still single-pass optimization |
| Single sort | O(n log n) | JavaScript sort algorithm |
| Multiple sorts | O(n log n × m) | m = number of sort criteria |
| Filter + Sort | O(n + n log n) | Filter first, then sort result |

### Efficiency Tips

1. **Use range filters** instead of chained string comparisons
2. **Order sorts by selectivity**: Least selective sort first (primary)
3. **Validate filters** before applying to large datasets
4. **Cache results** if applying same filters repeatedly
5. **Filter before sorting** for best performance

## Special Cases

### String Matching

- **Case-insensitive**: "ALICE", "alice", "Alice" all match
- **Substring match**: "alic" matches "alice"
- **Null values**: Treated as non-matching for string filters

### Date Handling

- **ISO strings**: "2024-01-15T10:30:00Z"
- **Timestamps**: Unix milliseconds (number)
- **Comparison**: Automatically parsed and compared as timestamps
- **Invalid dates**: Silently ignored in filters

### Numeric Amounts

- **String or number**: Both "1000" and 1000 work
- **Range comparison**: Numeric comparison after parsing
- **Invalid values**: NaN values excluded from results

### Sorting Null Values

- Null/undefined sort to **end of ascending** order
- Null/undefined sort to **end of descending** order

## Integration with Redux

For use with Redux state management:

```typescript
import { useSelector, useDispatch } from 'react-redux';
import { applyAdvancedFilters } from '@components';
import { setFilters } from '@state/slices/uiSlice';

export function FilteredTradesList() {
  const dispatch = useDispatch();
  const trades = useSelector(state => state.trades.allIds.map(id => state.trades.byId[id]));
  const filters = useSelector(state => state.ui.filters);
  const sortConfig = useSelector(state => state.ui.sortConfig);

  const filtered = applyAdvancedFilters(trades, filters, sortConfig);

  return (
    <div>
      {filtered.map(trade => (
        <TradeRow key={trade.id} trade={trade} />
      ))}
    </div>
  );
}
```

## Testing

Comprehensive test suite included in `filters.test.ts`:

- ✅ String filters (status, seller, buyer, tradeId)
- ✅ Range filters (amount, date)
- ✅ Boolean filters (isVerified, isDisputed)
- ✅ Multi-criteria combinations
- ✅ Single and multiple sort criteria
- ✅ Combined filter + sort operations
- ✅ Edge cases (empty arrays, null values, invalid data)
- ✅ Performance tests (10,000+ items)

Run tests with:

```bash
npm test -- filters.test.ts
```

## Error Handling

The function is defensive and forgiving:

- **Invalid filter values**: Gracefully skipped
- **Null/undefined data**: Safely handled
- **Type mismatches**: Gracefully coerced when possible
- **Empty input**: Returns empty array
- **Large datasets**: Efficient handling without errors

No exceptions are thrown under normal conditions.

## Browser Compatibility

- ✅ Modern browsers (Chrome, Firefox, Safari, Edge)
- ✅ Node.js 14+
- Uses: Array.filter(), Array.sort(), JSON parsing

## Future Enhancements

Potential additions (not currently implemented):

- [ ] Fuzzy matching for string filters
- [ ] OR logic for filter combinations
- [ ] Custom filter functions
- [ ] Result pagination
- [ ] Filter history/undo
- [ ] Export filtered results (CSV/JSON)

## Version History

- **v1.0.0** - Initial release
  - Basic filtering (strings, ranges, booleans)
  - Multi-criteria filtering
  - Complex sorting with multiple criteria
  - Performance optimizations for large datasets
