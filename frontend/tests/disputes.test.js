/**
 * Dispute Management Tests
 */

describe('Dispute Management', () => {
    
    beforeEach(() => {
        fetch.resetMocks();
    });

    describe('Raise Dispute', () => {
        test('should raise dispute with reason', async () => {
            fetch.mockResponseOnce(JSON.stringify({ id: 1 }));
            
            const result = await window.DisputeManager.raiseDispute(123, 'Item not received');
            
            expect(result.success).toBe(true);
            expect(result.disputeId).toBe(1);
            expect(fetch).toHaveBeenCalledWith(
                '/api/disputes/raise',
                expect.objectContaining({ method: 'POST' })
            );
        });

        test('should handle raise dispute error', async () => {
            fetch.mockResponseOnce('', { status: 400 });
            
            const result = await window.DisputeManager.raiseDispute(123, 'Issue');
            
            expect(result.success).toBe(false);
            expect(result.error).toBeDefined();
        });
    });

    describe('Get Dispute Details', () => {
        test('should fetch dispute details', async () => {
            const mockDispute = {
                id: 1,
                trade_id: 123,
                status: 'open',
                reason: 'Item not received',
                raised_by: 'GXXXXX...XXXXX'
            };
            
            fetch.mockResponseOnce(JSON.stringify(mockDispute));
            
            const result = await window.DisputeManager.getDisputeDetails(1);
            
            expect(result.success).toBe(true);
            expect(result.dispute.id).toBe(1);
            expect(fetch).toHaveBeenCalledWith('/api/disputes/1');
        });

        test('should handle fetch error', async () => {
            fetch.mockResponseOnce('', { status: 404 });
            
            const result = await window.DisputeManager.getDisputeDetails(999);
            
            expect(result.success).toBe(false);
        });
    });

    describe('Get Disputes by Trade', () => {
        test('should fetch disputes for trade', async () => {
            const mockDisputes = [
                { id: 1, trade_id: 123, status: 'open' },
                { id: 2, trade_id: 123, status: 'resolved' }
            ];
            
            fetch.mockResponseOnce(JSON.stringify(mockDisputes));
            
            const result = await window.DisputeManager.getDisputesByTrade(123);
            
            expect(result.success).toBe(true);
            expect(result.disputes.length).toBe(2);
            expect(fetch).toHaveBeenCalledWith('/api/disputes?trade_id=123');
        });
    });

    describe('Resolve Dispute', () => {
        test('should resolve dispute', async () => {
            const mockResolved = {
                id: 1,
                status: 'resolved',
                resolution: 'ReleaseToBuyer'
            };
            
            fetch.mockResponseOnce(JSON.stringify(mockResolved));
            
            const result = await window.DisputeManager.resolveDispute(
                1,
                'ReleaseToBuyer',
                'Buyer provided proof'
            );
            
            expect(result.success).toBe(true);
            expect(result.dispute.status).toBe('resolved');
            expect(fetch).toHaveBeenCalledWith(
                '/api/disputes/1/resolve',
                expect.objectContaining({
                    method: 'POST',
                    body: expect.stringContaining('ReleaseToBuyer')
                })
            );
        });
    });

    describe('Evidence Management', () => {
        test('should add valid evidence', () => {
            const file = new File(['content'], 'test.pdf', { type: 'application/pdf' });
            
            const result = window.DisputeManager.addEvidence(file);
            
            expect(result.success).toBe(true);
        });

        test('should reject oversized file', () => {
            const largeContent = new Array(6 * 1024 * 1024).fill('x').join('');
            const file = new File([largeContent], 'large.pdf', { type: 'application/pdf' });
            
            const result = window.DisputeManager.addEvidence(file);
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('5MB');
        });

        test('should reject unsupported file type', () => {
            const file = new File(['content'], 'test.exe', { type: 'application/x-msdownload' });
            
            const result = window.DisputeManager.addEvidence(file);
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('not allowed');
        });

        test('should remove evidence', () => {
            const file = new File(['content'], 'test.pdf', { type: 'application/pdf' });
            const result = window.DisputeManager.addEvidence(file);
            const evidenceId = result.evidenceId;
            
            window.DisputeManager.removeEvidence(evidenceId);
            
            const state = window.DisputeManager.getState();
            expect(state.evidence.length).toBe(0);
        });
    });

    describe('State Management', () => {
        test('should get dispute state', () => {
            const state = window.DisputeManager.getState();
            
            expect(state.disputes).toBeDefined();
            expect(state.currentDispute).toBeDefined();
            expect(state.evidence).toBeDefined();
            expect(state.filters).toBeDefined();
        });
    });
});
