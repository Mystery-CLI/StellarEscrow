/**
 * Quick Reference: Advanced Filters Utility
 * 
 * Usage examples for applyAdvancedFilters() function
 */

import { applyAdvancedFilters, validateFilters, FilterCriteria, SortConfig } from '@components';

// ============================================================================
// EXAMPLE 1: Simple Status Filter
// ============================================================================
function example1_simpleFilter() {
  const trades = [
    { id: '1', status: 'completed', amount: '1000' },
    { id: '2', status: 'funded', amount: '2000' },
    { id: '3', status: 'completed', amount: '1500' },
  ];

  const completed = applyAdvancedFilters(trades, { status: 'completed' });
  console.log(completed); // [trade-1, trade-3]
}

// ============================================================================
// EXAMPLE 2: Range Filtering (Amount)
// ============================================================================
function example2_rangeFilter() {
  const trades = [
    { id: '1', amount: '500', status: 'completed' },
    { id: '2', amount: '1500', status: 'funded' },
    { id: '3', amount: '3000', status: 'completed' },
  ];

  const filters: FilterCriteria = {
    minAmount: 1000,
    maxAmount: 2500,
  };

  const inRange = applyAdvancedFilters(trades, filters);
  console.log(inRange); // [trade-2]
}

// ============================================================================
// EXAMPLE 3: Date Range Filtering
// ============================================================================
function example3_dateRangeFilter() {
  const trades = [
    { id: '1', timestamp: '2024-01-05T10:00:00Z' },
    { id: '2', timestamp: '2024-01-15T10:00:00Z' },
    { id: '3', timestamp: '2024-01-25T10:00:00Z' },
  ];

  const filters: FilterCriteria = {
    startDate: '2024-01-10',
    endDate: '2024-01-20',
  };

  const janTrades = applyAdvancedFilters(trades, filters);
  console.log(janTrades); // [trade-2]
}

// ============================================================================
// EXAMPLE 4: Multi-Criteria Filtering (All must match)
// ============================================================================
function example4_multiCriteriaFilter() {
  const trades = [
    { id: '1', seller: 'alice', buyer: 'bob', amount: '1000', status: 'completed', isVerified: true },
    { id: '2', seller: 'alice', buyer: 'carol', amount: '2000', status: 'funded', isVerified: false },
    { id: '3', seller: 'bob', buyer: 'alice', amount: '500', status: 'completed', isVerified: true },
  ];

  const filters: FilterCriteria = {
    seller: 'alice',           // Must match seller name (substring)
    status: 'completed',       // Must have status
    minAmount: 900,            // Must be >= 900
    isVerified: true,          // Must be verified
  };

  const results = applyAdvancedFilters(trades, filters);
  console.log(results); // [trade-1]
}

// ============================================================================
// EXAMPLE 5: Simple Sorting
// ============================================================================
function example5_sorting() {
  const trades = [
    { id: '1', amount: '1500' },
    { id: '2', amount: '500' },
    { id: '3', amount: '2500' },
  ];

  // Sort by amount ascending
  const sortAsc: SortConfig[] = [{ key: 'amount', direction: 'asc' }];
  const sorted = applyAdvancedFilters(trades, {}, sortAsc);
  console.log(sorted); // [500, 1500, 2500]

  // Sort by amount descending
  const sortDesc: SortConfig[] = [{ key: 'amount', direction: 'desc' }];
  const reverse = applyAdvancedFilters(trades, {}, sortDesc);
  console.log(reverse); // [2500, 1500, 500]
}

// ============================================================================
// EXAMPLE 6: Multiple Sort Criteria (Primary + Secondary)
// ============================================================================
function example6_multiSort() {
  const trades = [
    { id: '1', status: 'completed', amount: '2000' },
    { id: '2', status: 'funded', amount: '500' },
    { id: '3', status: 'completed', amount: '1000' },
    { id: '4', status: 'funded', amount: '3000' },
  ];

  // Primary: by status ascending (completed < funded)
  // Secondary: by amount ascending (within same status)
  const sortConfig: SortConfig[] = [
    { key: 'status', direction: 'asc' },
    { key: 'amount', direction: 'asc' },
  ];

  const sorted = applyAdvancedFilters(trades, {}, sortConfig);
  // Result: completed (1000), completed (2000), funded (500), funded (3000)
}

// ============================================================================
// EXAMPLE 7: Filter + Sort Combined
// ============================================================================
function example7_filterAndSort() {
  const trades = [
    { id: '1', status: 'completed', amount: '1000', timestamp: '2024-01-20' },
    { id: '2', status: 'funded', amount: '2000', timestamp: '2024-01-10' },
    { id: '3', status: 'completed', amount: '1500', timestamp: '2024-01-15' },
    { id: '4', status: 'completed', amount: '500', timestamp: '2024-01-25' },
  ];

  const filters: FilterCriteria = {
    status: 'completed',
    minAmount: 750,
  };

  const sortConfig: SortConfig[] = [
    { key: 'timestamp', direction: 'desc' }, // Most recent first
  ];

  const results = applyAdvancedFilters(trades, filters, sortConfig);
  // [trade-4 (500, 2024-01-25)]
  // [trade-1 (1000, 2024-01-20)]
  // [trade-3 (1500, 2024-01-15)]
}

// ============================================================================
// EXAMPLE 8: Using with Redux
// ============================================================================
function example8_withRedux() {
  // In a React component using Redux
  // import { useSelector } from 'react-redux';

  // const trades = useSelector(state => 
  //   state.trades.allIds.map(id => state.trades.byId[id])
  // );
  // const filters = useSelector(state => state.ui.filters);
  // const sortConfig = useSelector(state => state.ui.sortConfig || []);

  // const filteredTrades = applyAdvancedFilters(trades, filters, sortConfig);

  // return (
  //   <div>
  //     {filteredTrades.map(trade => <TradeCard key={trade.id} trade={trade} />)}
  //   </div>
  // );
}

// ============================================================================
// EXAMPLE 9: Boolean Filters (isVerified, isDisputed)
// ============================================================================
function example9_booleanFilters() {
  const trades = [
    { id: '1', status: 'completed', isVerified: true, isDisputed: false },
    { id: '2', status: 'funded', isVerified: false, isDisputed: false },
    { id: '3', status: 'disputed', isVerified: true, isDisputed: true },
  ];

  // Get verified trades that are NOT disputed
  const filters: FilterCriteria = {
    isVerified: true,
    isDisputed: false,
  };

  const safeTrades = applyAdvancedFilters(trades, filters);
  console.log(safeTrades); // [trade-1]
}

// ============================================================================
// EXAMPLE 10: Substring Matching (Sellers, Buyers, Arbitrators)
// ============================================================================
function example10_substringMatch() {
  const trades = [
    { id: '1', seller: 'alice-smith', buyer: 'bob-jones' },
    { id: '2', seller: 'alice-brown', buyer: 'carol-white' },
    { id: '3', seller: 'dave-green', buyer: 'alice-davis' },
  ];

  // Find all trades involving someone named "alice" (case-insensitive)
  // This will match: alice-smith (seller), alice-brown (seller), alice-davis (buyer)
  
  // Note: applyAdvancedFilters uses AND logic, so this would need separated calls:
  const byAliceSeller = applyAdvancedFilters(trades, { seller: 'alice' });
  console.log(byAliceSeller); // [trade-1, trade-2]

  // For OR logic, you'd need to call separately and merge results
}

// ============================================================================
// EXAMPLE 11: Empty Filters (Get All)
// ============================================================================
function example11_noFilters() {
  const trades = [
    { id: '1', status: 'completed' },
    { id: '2', status: 'funded' },
  ];

  // No filters = returns all
  const all = applyAdvancedFilters(trades, {});
  console.log(all); // [trade-1, trade-2]

  // No filters, no sort = returns original array
  const same = applyAdvancedFilters(trades);
  console.log(same); // [trade-1, trade-2] (same reference to original? No, creates new sorted array)
}

// ============================================================================
// EXAMPLE 12: Filter Validation
// ============================================================================
function example12_validation() {
  const filters: FilterCriteria = {
    minAmount: 5000,
    maxAmount: 1000, // Invalid! maxAmount < minAmount
  };

  if (validateFilters(filters)) {
    console.log('Filters are valid');
  } else {
    console.warn('Invalid filter configuration detected');
  }
}

// ============================================================================
// EXAMPLE 13: Large Dataset Performance
// ============================================================================
function example13_largeDataset() {
  // Generate 10,000 trades
  const trades = Array.from({ length: 10000 }, (_, i) => ({
    id: `trade-${i}`,
    seller: `seller-${i % 100}`,
    amount: String((i % 5000) + 1),
    status: ['completed', 'funded', 'disputed'][i % 3],
    timestamp: new Date(2024, 0, (i % 365) + 1).toISOString(),
    isVerified: i % 2 === 0,
  }));

  const filters: FilterCriteria = {
    status: 'completed',
    minAmount: 500,
    maxAmount: 3000,
    isVerified: true,
  };

  const sortConfig: SortConfig[] = [
    { key: 'timestamp', direction: 'desc' },
    { key: 'amount', direction: 'asc' },
  ];

  const start = performance.now();
  const results = applyAdvancedFilters(trades, filters, sortConfig);
  const end = performance.now();

  console.log(`Filtered and sorted ${trades.length} trades in ${end - start}ms`);
  console.log(`Found ${results.length} matching trades`);
}

// ============================================================================
// EXAMPLE 14: Custom Numeric Field Filtering
// ============================================================================
function example14_customFields() {
  const trades = [
    { id: '1', amount: '1000', feePercent: 2, trustScore: 95 },
    { id: '2', amount: '2000', feePercent: 1.5, trustScore: 87 },
    { id: '3', amount: '1500', feePercent: 2.5, trustScore: 92 },
  ];

  // Note: Custom numeric range filters would need to be added to FilterCriteria
  // For now, use the basic filters and filter the result manually:
  
  const base = applyAdvancedFilters(trades, { minAmount: 1000 });
  const custom = base.filter(t => t.feePercent <= 2 && t.trustScore > 90);
  
  console.log(custom); // Trades with amount >= 1000, fee <= 2%, trust > 90
}

// ============================================================================
// EXAMPLE 15: Date Filtering with Timestamps
// ============================================================================
function example15_dateWithTimestamps() {
  const trades = [
    { id: '1', timestamp: 1705334400000 }, // 2024-01-15
    { id: '2', timestamp: 1705507200000 }, // 2024-01-17
    { id: '3', timestamp: 1705593600000 }, // 2024-01-18
  ];

  // Can use timestamps (milliseconds) directly
  const filters: FilterCriteria = {
    startDate: 1705420800000, // 2024-01-16
    endDate: 1705593600000,   // 2024-01-18
  };

  const inRange = applyAdvancedFilters(trades, filters);
  console.log(inRange); // [trade-2, trade-3]

  // Or use ISO strings (more readable)
  const filters2: FilterCriteria = {
    startDate: '2024-01-16',
    endDate: '2024-01-18',
  };

  const inRange2 = applyAdvancedFilters(trades, filters2);
  console.log(inRange2); // Same result
}

export {
  example1_simpleFilter,
  example2_rangeFilter,
  example3_dateRangeFilter,
  example4_multiCriteriaFilter,
  example5_sorting,
  example6_multiSort,
  example7_filterAndSort,
  example8_withRedux,
  example9_booleanFilters,
  example10_substringMatch,
  example11_noFilters,
  example12_validation,
  example13_largeDataset,
  example14_customFields,
  example15_dateWithTimestamps,
};
