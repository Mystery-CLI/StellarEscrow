/**
 * Advanced filtering and sorting utility for StellarEscrow trades
 * Efficiently handles multi-criteria filtering and complex sorting operations
 */

/**
 * Configuration for a single sort criterion
 */
export interface SortConfig {
  key: string; // Field name to sort by (e.g., 'createdAt', 'amount', 'status')
  direction: 'asc' | 'desc'; // Sort direction
}

/**
 * Filter criteria supporting various data types
 * - String filters: exact match or substring (status, seller, buyer, etc.)
 * - Range filters: { min, max } for numeric/date fields (amount, timestamp)
 * - Boolean filters: true/false for verification status, etc.
 */
export interface FilterCriteria {
  status?: string; // Exact match or contains
  seller?: string; // Exact match or contains
  buyer?: string; // Exact match or contains
  arbitrator?: string; // Exact match or contains
  tradeId?: string; // Exact match

  // Range filters for amounts
  minAmount?: number | string;
  maxAmount?: number | string;

  // Range filters for dates (ISO string or timestamp)
  startDate?: string | number;
  endDate?: string | number;

  // Boolean and verification filters
  isVerified?: boolean;
  isDisputed?: boolean;
  [key: string]: any; // Allow custom filters
}

/**
 * Applies advanced filtering and sorting to a dataset
 *
 * @param data - Array of trade objects to filter and sort
 * @param filters - Filter criteria object with optional fields
 * @param sortConfig - Array of sort configurations applied in order
 * @returns Filtered and sorted array
 *
 * @example
 * ```typescript
 * const trades = [...];
 * const filters: FilterCriteria = {
 *   status: 'completed',
 *   minAmount: 100,
 *   maxAmount: 5000,
 *   startDate: '2024-01-01'
 * };
 * const sortConfig: SortConfig[] = [
 *   { key: 'createdAt', direction: 'desc' },
 *   { key: 'amount', direction: 'asc' }
 * ];
 *
 * const results = applyAdvancedFilters(trades, filters, sortConfig);
 * ```
 */
export function applyAdvancedFilters<T extends Record<string, any>>(
  data: T[],
  filters: Partial<FilterCriteria> = {},
  sortConfig: SortConfig[] = []
): T[] {
  if (!Array.isArray(data)) {
    console.warn('applyAdvancedFilters: data must be an array');
    return [];
  }

  // Early exit optimization: if no filters and no sorting, return original data
  if (Object.keys(filters).length === 0 && sortConfig.length === 0) {
    return data;
  }

  // Step 1: Apply filters efficiently using a single pass
  let filtered = filterData(data, filters);

  // Step 2: Apply sorting (can be multiple criteria)
  if (sortConfig.length > 0) {
    filtered = sortData(filtered, sortConfig);
  }

  return filtered;
}

/**
 * Filters data based on criteria
 * Optimized to use a single pass through the array
 */
function filterData<T extends Record<string, any>>(
  data: T[],
  filters: Partial<FilterCriteria>
): T[] {
  return data.filter((item) => {
    // String filters (case-insensitive substring match)
    if (filters.status !== undefined && !matchStringFilter(item.status, filters.status)) {
      return false;
    }

    if (filters.seller !== undefined && !matchStringFilter(item.seller, filters.seller)) {
      return false;
    }

    if (filters.buyer !== undefined && !matchStringFilter(item.buyer, filters.buyer)) {
      return false;
    }

    if (
      filters.arbitrator !== undefined &&
      !matchStringFilter(item.arbitrator, filters.arbitrator)
    ) {
      return false;
    }

    if (filters.tradeId !== undefined && item.id !== filters.tradeId) {
      return false;
    }

    // Range filters for amount (numeric)
    if (filters.minAmount !== undefined) {
      const amount = parseFloat(String(item.amount || 0));
      const minAmount = parseFloat(String(filters.minAmount));
      if (isNaN(amount) || amount < minAmount) {
        return false;
      }
    }

    if (filters.maxAmount !== undefined) {
      const amount = parseFloat(String(item.amount || 0));
      const maxAmount = parseFloat(String(filters.maxAmount));
      if (isNaN(amount) || amount > maxAmount) {
        return false;
      }
    }

    // Range filters for dates
    if (filters.startDate !== undefined) {
      const timestamp = parseDate(item.timestamp);
      const startDate = parseDate(filters.startDate);
      if (timestamp === null || startDate === null || timestamp < startDate) {
        return false;
      }
    }

    if (filters.endDate !== undefined) {
      const timestamp = parseDate(item.timestamp);
      const endDate = parseDate(filters.endDate);
      if (timestamp === null || endDate === null || timestamp > endDate) {
        return false;
      }
    }

    // Boolean filters
    if (filters.isVerified !== undefined && item.isVerified !== filters.isVerified) {
      return false;
    }

    if (filters.isDisputed !== undefined && item.isDisputed !== filters.isDisputed) {
      return false;
    }

    // Custom field filters (boolean check)
    for (const [key, value] of Object.entries(filters)) {
      // Skip known fields already processed
      if (
        [
          'status',
          'seller',
          'buyer',
          'arbitrator',
          'tradeId',
          'minAmount',
          'maxAmount',
          'startDate',
          'endDate',
          'isVerified',
          'isDisputed',
        ].includes(key)
      ) {
        continue;
      }

      // Handle custom filters
      if (typeof value === 'boolean' && item[key] !== value) {
        return false;
      }
    }

    return true;
  });
}

/**
 * Sorts data based on multiple sort criteria
 * Supports chained sorting with primary and secondary sort keys
 */
function sortData<T extends Record<string, any>>(
  data: T[],
  sortConfig: SortConfig[]
): T[] {
  // Create a copy to avoid mutating original array
  const sorted = [...data];

  // Apply sorts in reverse order (secondary sorts first)
  // This ensures primary sort takes precedence
  for (let i = sortConfig.length - 1; i >= 0; i--) {
    const { key, direction } = sortConfig[i];
    const multiplier = direction === 'asc' ? 1 : -1;

    sorted.sort((a, b) => {
      const aVal = a[key];
      const bVal = b[key];

      // Handle null/undefined values
      if (aVal === null || aVal === undefined) return multiplier;
      if (bVal === null || bVal === undefined) return -multiplier;

      // String comparison (case-insensitive)
      if (typeof aVal === 'string' && typeof bVal === 'string') {
        return multiplier * aVal.localeCompare(bVal, undefined, { numeric: true });
      }

      // Numeric comparison
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return multiplier * (aVal - bVal);
      }

      // Date comparison (ISO strings or timestamps)
      if (
        (typeof aVal === 'string' && /^\d{4}-\d{2}-\d{2}/.test(aVal)) ||
        typeof aVal === 'number'
      ) {
        const aTime = parseDate(aVal);
        const bTime = parseDate(bVal);
        if (aTime !== null && bTime !== null) {
          return multiplier * (aTime - bTime);
        }
      }

      // Fallback comparison
      if (aVal > bVal) return multiplier;
      if (aVal < bVal) return -multiplier;
      return 0;
    });
  }

  return sorted;
}

/**
 * Matches a string value against a filter (case-insensitive substring match)
 */
function matchStringFilter(value: any, filter: string): boolean {
  if (value === undefined || value === null) {
    return false;
  }

  const valueStr = String(value).toLowerCase();
  const filterStr = String(filter).toLowerCase();

  return valueStr.includes(filterStr);
}

/**
 * Parses a date value (ISO string or timestamp number)
 * Returns milliseconds since epoch, or null if invalid
 */
function parseDate(value: any): number | null {
  if (typeof value === 'number') {
    // Assume it's already a timestamp in milliseconds
    return Number.isFinite(value) ? value : null;
  }

  if (typeof value === 'string') {
    // Try ISO date parsing
    const time = new Date(value).getTime();
    return Number.isFinite(time) ? time : null;
  }

  return null;
}

/**
 * Validates filter criteria (optional utility)
 */
export function validateFilters(filters: Partial<FilterCriteria>): boolean {
  if (filters.minAmount !== undefined && filters.maxAmount !== undefined) {
    const min = parseFloat(String(filters.minAmount));
    const max = parseFloat(String(filters.maxAmount));
    if (!isNaN(min) && !isNaN(max) && min > max) {
      console.warn('Filter validation: minAmount should not exceed maxAmount');
      return false;
    }
  }

  if (filters.startDate !== undefined && filters.endDate !== undefined) {
    const start = parseDate(filters.startDate);
    const end = parseDate(filters.endDate);
    if (start !== null && end !== null && start > end) {
      console.warn('Filter validation: startDate should not exceed endDate');
      return false;
    }
  }

  return true;
}
