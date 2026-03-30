import {
  applyAdvancedFilters,
  validateFilters,
  FilterCriteria,
  SortConfig,
} from './filters';

describe('applyAdvancedFilters', () => {
  // Sample trade data for testing
  const sampleTrades: any[] = [
    {
      id: 'trade-1',
      seller: 'alice',
      buyer: 'bob',
      amount: '1000',
      status: 'completed',
      arbitrator: 'charlie',
      timestamp: '2024-01-15T10:30:00Z',
      isVerified: true,
      isDisputed: false,
    },
    {
      id: 'trade-2',
      seller: 'bob',
      buyer: 'carol',
      amount: '2500',
      status: 'funded',
      arbitrator: null,
      timestamp: '2024-01-10T14:20:00Z',
      isVerified: true,
      isDisputed: false,
    },
    {
      id: 'trade-3',
      seller: 'carol',
      buyer: 'alice',
      amount: '500',
      status: 'disputed',
      arbitrator: 'dave',
      timestamp: '2024-01-20T09:15:00Z',
      isVerified: false,
      isDisputed: true,
    },
    {
      id: 'trade-4',
      seller: 'dave',
      buyer: 'alice',
      amount: '1500',
      status: 'created',
      arbitrator: null,
      timestamp: '2024-01-05T12:00:00Z',
      isVerified: true,
      isDisputed: false,
    },
  ];

  describe('String Filters', () => {
    it('should filter by status', () => {
      const filters: FilterCriteria = { status: 'completed' };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-1');
    });

    it('should filter by seller with case-insensitive substring match', () => {
      const filters: FilterCriteria = { seller: 'alic' };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-1');
    });

    it('should filter by buyer', () => {
      const filters: FilterCriteria = { buyer: 'bob' };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-1');
    });

    it('should filter by tradeId (exact match)', () => {
      const filters: FilterCriteria = { tradeId: 'trade-2' };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-2');
    });
  });

  describe('Range Filters', () => {
    it('should filter by amount range (min and max)', () => {
      const filters: FilterCriteria = { minAmount: 600, maxAmount: 2000 };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(2);
      expect(result.map((t) => t.id)).toContain('trade-1');
      expect(result.map((t) => t.id)).toContain('trade-4');
    });

    it('should filter by minimum amount only', () => {
      const filters: FilterCriteria = { minAmount: 1500 };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(2);
      expect(result.map((t) => t.id)).toContain('trade-2');
      expect(result.map((t) => t.id)).toContain('trade-4');
    });

    it('should filter by maximum amount only', () => {
      const filters: FilterCriteria = { maxAmount: 1000 };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(2);
      expect(result.map((t) => t.id)).toContain('trade-1');
      expect(result.map((t) => t.id)).toContain('trade-3');
    });

    it('should filter by date range', () => {
      const filters: FilterCriteria = {
        startDate: '2024-01-10',
        endDate: '2024-01-20',
      };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(3);
      expect(result.map((t) => t.id)).toContain('trade-1');
      expect(result.map((t) => t.id)).toContain('trade-2');
      expect(result.map((t) => t.id)).toContain('trade-3');
    });

    it('should filter by start date only', () => {
      const filters: FilterCriteria = { startDate: '2024-01-15' };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(2);
    });
  });

  describe('Boolean Filters', () => {
    it('should filter by isVerified', () => {
      const filters: FilterCriteria = { isVerified: true };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(3);
      expect(result.every((t) => t.isVerified === true)).toBe(true);
    });

    it('should filter by isDisputed', () => {
      const filters: FilterCriteria = { isDisputed: true };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-3');
    });

    it('should filter by multiple boolean conditions', () => {
      const filters: FilterCriteria = { isVerified: true, isDisputed: false };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(3);
      expect(result.every((t) => t.isVerified === true && t.isDisputed === false)).toBe(true);
    });
  });

  describe('Multi-Criteria Filters', () => {
    it('should apply multiple filter conditions (AND logic)', () => {
      const filters: FilterCriteria = {
        status: 'completed',
        isVerified: true,
        minAmount: 500,
      };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-1');
    });

    it('should filter by status and amount range', () => {
      const filters: FilterCriteria = {
        status: 'funded',
        maxAmount: 3000,
      };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-2');
    });
  });

  describe('Sorting', () => {
    it('should sort by single field ascending', () => {
      const sortConfig: SortConfig[] = [{ key: 'amount', direction: 'asc' }];
      const result = applyAdvancedFilters(sampleTrades, {}, sortConfig);
      expect(result.map((t) => parseInt(t.amount))).toEqual([500, 1000, 1500, 2500]);
    });

    it('should sort by single field descending', () => {
      const sortConfig: SortConfig[] = [{ key: 'amount', direction: 'desc' }];
      const result = applyAdvancedFilters(sampleTrades, {}, sortConfig);
      expect(result.map((t) => parseInt(t.amount))).toEqual([2500, 1500, 1000, 500]);
    });

    it('should sort by timestamp (date) descending', () => {
      const sortConfig: SortConfig[] = [{ key: 'timestamp', direction: 'desc' }];
      const result = applyAdvancedFilters(sampleTrades, {}, sortConfig);
      expect(result.map((t) => t.id)).toEqual(['trade-3', 'trade-1', 'trade-2', 'trade-4']);
    });

    it('should sort by status (string) ascending', () => {
      const sortConfig: SortConfig[] = [{ key: 'status', direction: 'asc' }];
      const result = applyAdvancedFilters(sampleTrades, {}, sortConfig);
      expect(result.map((t) => t.status)).toEqual(['completed', 'created', 'disputed', 'funded']);
    });

    it('should support multiple sort criteria (primary and secondary)', () => {
      const sortConfig: SortConfig[] = [
        { key: 'isVerified', direction: 'desc' }, // Primary: verified first
        { key: 'amount', direction: 'desc' }, // Secondary: highest amount first
      ];
      const result = applyAdvancedFilters(sampleTrades, {}, sortConfig);
      // Verified trades sorted by amount desc, then unverified
      expect(result[0].isVerified).toBe(true); // Should be in first group
      expect(result[result.length - 1].isVerified).toBe(false);
    });

    it('should handle null/undefined values when sorting', () => {
      const testData = [
        { id: '1', status: 'completed', arbitrator: 'alice' },
        { id: '2', status: 'funded', arbitrator: null },
        { id: '3', status: 'disputed', arbitrator: 'bob' },
      ];
      const sortConfig: SortConfig[] = [{ key: 'arbitrator', direction: 'asc' }];
      expect(() => applyAdvancedFilters(testData, {}, sortConfig)).not.toThrow();
    });
  });

  describe('Filters + Sorting Combined', () => {
    it('should filter and then sort results', () => {
      const filters: FilterCriteria = { isVerified: true };
      const sortConfig: SortConfig[] = [{ key: 'amount', direction: 'desc' }];
      const result = applyAdvancedFilters(sampleTrades, filters, sortConfig);
      expect(result).toHaveLength(3);
      expect(result.map((t) => parseInt(t.amount))).toEqual([2500, 1500, 1000]);
    });

    it('should filter by status and sort by date descending', () => {
      const filters: FilterCriteria = { status: 'completed' };
      const sortConfig: SortConfig[] = [{ key: 'timestamp', direction: 'desc' }];
      const result = applyAdvancedFilters(sampleTrades, filters, sortConfig);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('trade-1');
    });

    it('should filter by range and sort by status', () => {
      const filters: FilterCriteria = { minAmount: 1000 };
      const sortConfig: SortConfig[] = [{ key: 'status', direction: 'asc' }];
      const result = applyAdvancedFilters(sampleTrades, filters, sortConfig);
      expect(result.length).toBeGreaterThan(0);
      expect(result[0].status).toBe('completed');
    });
  });

  describe('Edge Cases', () => {
    it('should return empty array for empty input', () => {
      const result = applyAdvancedFilters([], {});
      expect(result).toEqual([]);
    });

    it('should return original array when no filters and no sorts applied', () => {
      const result = applyAdvancedFilters(sampleTrades, {}, []);
      expect(result).toHaveLength(sampleTrades.length);
    });

    it('should return empty array when no matches found', () => {
      const filters: FilterCriteria = { status: 'nonexistent' };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result).toEqual([]);
    });

    it('should not mutate original array', () => {
      const original = [...sampleTrades];
      const sortConfig: SortConfig[] = [{ key: 'amount', direction: 'asc' }];
      applyAdvancedFilters(sampleTrades, {}, sortConfig);
      expect(sampleTrades).toEqual(original);
    });

    it('should handle numeric amounts as strings', () => {
      const filters: FilterCriteria = { minAmount: '1000' };
      const result = applyAdvancedFilters(sampleTrades, filters);
      expect(result.length).toBeGreaterThan(0);
    });

    it('should ignore invalid filters gracefully', () => {
      const filters: FilterCriteria = {
        status: 'completed',
        invalidField: 'should be ignored',
      };
      expect(() => applyAdvancedFilters(sampleTrades, filters)).not.toThrow();
    });
  });

  describe('Performance', () => {
    it('should handle large datasets efficiently', () => {
      // Generate 10,000 trades
      const largeTrades = Array.from({ length: 10000 }, (_, i) => ({
        id: `trade-${i}`,
        seller: `seller-${i % 100}`,
        buyer: `buyer-${i % 100}`,
        amount: String((i % 5000) + 1),
        status: ['completed', 'funded', 'disputed', 'created'][i % 4],
        timestamp: new Date(2024, 0, (i % 30) + 1).toISOString(),
        isVerified: i % 2 === 0,
        isDisputed: i % 10 === 0,
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

      const startTime = performance.now();
      const result = applyAdvancedFilters(largeTrades, filters, sortConfig);
      const endTime = performance.now();

      // Should complete in reasonable time (< 500ms for 10k items)
      expect(endTime - startTime).toBeLessThan(500);
      expect(result.length).toBeGreaterThan(0);
      expect(result[0].status).toBe('completed');
    });
  });
});

describe('validateFilters', () => {
  it('should validate that minAmount does not exceed maxAmount', () => {
    const filters: FilterCriteria = { minAmount: 5000, maxAmount: 1000 };
    const result = validateFilters(filters);
    expect(result).toBe(false);
  });

  it('should validate that startDate does not exceed endDate', () => {
    const filters: FilterCriteria = {
      startDate: '2024-12-31',
      endDate: '2024-01-01',
    };
    const result = validateFilters(filters);
    expect(result).toBe(false);
  });

  it('should return true for valid filters', () => {
    const filters: FilterCriteria = {
      minAmount: 100,
      maxAmount: 5000,
      startDate: '2024-01-01',
      endDate: '2024-12-31',
    };
    const result = validateFilters(filters);
    expect(result).toBe(true);
  });

  it('should return true for empty filters', () => {
    const result = validateFilters({});
    expect(result).toBe(true);
  });
});
