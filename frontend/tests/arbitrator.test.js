/**
 * Arbitrator Dashboard Tests
 */

describe('Arbitrator Dashboard', () => {
    
    beforeEach(() => {
        fetch.resetMocks();
    });

    describe('Arbitrator Status', () => {
        test('should check arbitrator status', async () => {
            fetch.mockResponseOnce(JSON.stringify({ is_arbitrator: true }));
            
            const result = await window.ArbitratorDashboard.checkArbitratorStatus('GXXXXX...XXXXX');
            
            expect(result.success).toBe(true);
            expect(result.isArbitrator).toBe(true);
        });

        test('should handle non-arbitrator', async () => {
            fetch.mockResponseOnce(JSON.stringify({ is_arbitrator: false }));
            
            const result = await window.ArbitratorDashboard.checkArbitratorStatus('GXXXXX...XXXXX');
            
            expect(result.success).toBe(true);
            expect(result.isArbitrator).toBe(false);
        });
    });

    describe('Assigned Disputes', () => {
        test('should fetch assigned disputes', async () => {
            const mockDisputes = [
                { id: 1, trade_id: 123, status: 'open', reason: 'Issue' },
                { id: 2, trade_id: 124, status: 'pending', reason: 'Problem' }
            ];
            
            fetch.mockResponseOnce(JSON.stringify(mockDisputes));
            
            const result = await window.ArbitratorDashboard.getAssignedDisputes();
            
            expect(result.success).toBe(true);
            expect(result.disputes.length).toBe(2);
            expect(fetch).toHaveBeenCalledWith('/api/arbitrators/disputes');
        });

        test('should handle fetch error', async () => {
            fetch.mockResponseOnce('', { status: 500 });
            
            const result = await window.ArbitratorDashboard.getAssignedDisputes();
            
            expect(result.success).toBe(false);
        });
    });

    describe('Arbitrator Statistics', () => {
        test('should fetch arbitrator stats', async () => {
            const mockStats = {
                totalDisputes: 10,
                resolvedDisputes: 8,
                pendingDisputes: 2,
                resolutionRate: 80,
                avgResolutionTime: 3,
                buyerReleaseCount: 5,
                sellerReleaseCount: 3
            };
            
            fetch.mockResponseOnce(JSON.stringify(mockStats));
            
            const result = await window.ArbitratorDashboard.getArbitratorStats('GXXXXX...XXXXX');
            
            expect(result.success).toBe(true);
            expect(result.stats.totalDisputes).toBe(10);
            expect(result.stats.resolvedDisputes).toBe(8);
        });
    });

    describe('Performance Metrics', () => {
        test('should fetch performance metrics', async () => {
            const mockMetrics = {
                rating: 4.8,
                reviewCount: 25,
                joinDate: '2024-01-01',
                totalVolume: 50000
            };
            
            fetch.mockResponseOnce(JSON.stringify(mockMetrics));
            
            const result = await window.ArbitratorDashboard.getPerformanceMetrics('GXXXXX...XXXXX');
            
            expect(result.success).toBe(true);
            expect(result.metrics.rating).toBe(4.8);
            expect(result.metrics.reviewCount).toBe(25);
        });
    });

    describe('Resolution Submission', () => {
        test('should submit resolution', async () => {
            const mockResolved = {
                id: 1,
                status: 'resolved',
                resolution: 'ReleaseToBuyer'
            };
            
            fetch.mockResponseOnce(JSON.stringify(mockResolved));
            
            const result = await window.ArbitratorDashboard.submitResolution(
                1,
                'ReleaseToBuyer',
                'Buyer provided proof'
            );
            
            expect(result.success).toBe(true);
            expect(result.dispute.status).toBe('resolved');
            expect(fetch).toHaveBeenCalledWith(
                '/api/arbitrators/disputes/1/resolve',
                expect.objectContaining({ method: 'POST' })
            );
        });

        test('should handle resolution error', async () => {
            fetch.mockResponseOnce('', { status: 400 });
            
            const result = await window.ArbitratorDashboard.submitResolution(1, 'ReleaseToBuyer');
            
            expect(result.success).toBe(false);
        });
    });

    describe('State Management', () => {
        test('should get dashboard state', () => {
            const state = window.ArbitratorDashboard.getState();
            
            expect(state.isArbitrator).toBeDefined();
            expect(state.assignedDisputes).toBeDefined();
            expect(state.statistics).toBeDefined();
            expect(state.performanceMetrics).toBeDefined();
        });
    });
});
