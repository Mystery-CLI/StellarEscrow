/**
 * UAT Scenario: Compliance User Experience
 *
 * Stakeholder: Compliance Officer / Platform Admin
 * Goal: Verify compliance checks are transparent, reviewable, and reportable.
 *
 * Acceptance Criteria:
 *   AC-5: Compliance status is queryable per address
 *   AC-6: Manual review override is available to admins
 *   AC-7: Compliance reports are generated for date ranges
 *   AC-8: Blocked addresses are clearly identified
 */

import { api, TEST_ADDRESSES } from '../setup/uat-client';

describe('UAT — Compliance UX (AC-5 through AC-8)', () => {
  describe('AC-5: Compliance status visibility', () => {
    it('compliance status endpoint returns a response for any address', async () => {
      const res = await api.get(`/compliance/status/${TEST_ADDRESSES.seller}`);
      expect(res.status).toBe(200);
      expect(res.data).toHaveProperty('status');
    });

    it('compliance status includes risk_score when a check exists', async () => {
      const res = await api.get(`/compliance/status/${TEST_ADDRESSES.seller}`);
      if (res.data.status !== 'not_found') {
        expect(typeof res.data.risk_score).toBe('number');
        expect(res.data.risk_score).toBeGreaterThanOrEqual(0);
        expect(res.data.risk_score).toBeLessThanOrEqual(100);
      }
    });
  });

  describe('AC-7: Compliance reporting', () => {
    it('compliance report endpoint returns required fields', async () => {
      const from = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString();
      const to   = new Date().toISOString();
      const res  = await api.get(`/compliance/report?from=${from}&to=${to}`);

      expect(res.status).toBe(200);
      expect(res.data).toHaveProperty('total_checks');
      expect(res.data).toHaveProperty('approved');
      expect(res.data).toHaveProperty('blocked');
      expect(res.data).toHaveProperty('avg_risk_score');
      expect(res.data).toHaveProperty('generated_at');
    });

    it('compliance report counts are non-negative integers', async () => {
      const res = await api.get('/compliance/report');
      expect(res.status).toBe(200);
      expect(res.data.total_checks).toBeGreaterThanOrEqual(0);
      expect(res.data.approved).toBeGreaterThanOrEqual(0);
      expect(res.data.blocked).toBeGreaterThanOrEqual(0);
    });
  });

  describe('AC-8: Blocked address identification', () => {
    it('compliance report exposes blocked count', async () => {
      const res = await api.get('/compliance/report');
      expect(res.status).toBe(200);
      expect(typeof res.data.blocked).toBe('number');
    });
  });
});
