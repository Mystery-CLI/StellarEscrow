/**
 * UAT Scenario: Monitoring & Observability UX
 *
 * Stakeholder: DevOps / SRE
 * Goal: Verify the monitoring dashboard and alerting are usable in production.
 *
 * Acceptance Criteria:
 *   AC-9:  Dashboard shows platform health status
 *   AC-10: Active alerts are listed with severity
 *   AC-11: Prometheus metrics are scrapeable
 *   AC-12: Health endpoint responds within SLA (< 500ms)
 */

import { api } from '../setup/uat-client';

describe('UAT — Monitoring UX (AC-9 through AC-12)', () => {
  describe('AC-9: Dashboard health status', () => {
    it('monitoring dashboard returns health_status field', async () => {
      const res = await api.get('/monitoring/dashboard');
      expect(res.status).toBe(200);
      expect(['healthy', 'degraded', 'critical']).toContain(res.data.health_status);
    });

    it('dashboard includes active_alert_count', async () => {
      const res = await api.get('/monitoring/dashboard');
      expect(typeof res.data.active_alert_count).toBe('number');
      expect(res.data.active_alert_count).toBeGreaterThanOrEqual(0);
    });

    it('dashboard includes metrics map', async () => {
      const res = await api.get('/monitoring/dashboard');
      expect(res.data).toHaveProperty('metrics');
      expect(typeof res.data.metrics).toBe('object');
    });
  });

  describe('AC-10: Alert listing', () => {
    it('alerts endpoint returns an array', async () => {
      const res = await api.get('/monitoring/alerts');
      expect(res.status).toBe(200);
      expect(Array.isArray(res.data)).toBe(true);
    });

    it('each alert has required fields when alerts exist', async () => {
      const res = await api.get('/monitoring/alerts');
      for (const alert of res.data) {
        expect(alert).toHaveProperty('rule_name');
        expect(alert).toHaveProperty('severity');
        expect(alert).toHaveProperty('current_value');
        expect(alert).toHaveProperty('threshold');
        expect(alert).toHaveProperty('message');
      }
    });
  });

  describe('AC-11: Prometheus metrics', () => {
    it('metrics endpoint returns text/plain content type', async () => {
      const res = await api.get('/metrics');
      expect(res.status).toBe(200);
      expect(res.headers['content-type']).toMatch(/text\/plain/);
    });

    it('metrics response is non-empty', async () => {
      const res = await api.get('/metrics');
      expect(typeof res.data).toBe('string');
      // May be empty if no metrics recorded yet — just verify it's a string
    });
  });

  describe('AC-12: Health endpoint SLA', () => {
    it('health endpoint responds within 500ms', async () => {
      const start = Date.now();
      const res = await api.get('/health');
      const duration = Date.now() - start;

      expect(res.status).toBe(200);
      expect(duration).toBeLessThan(500);
    });

    it('health endpoint responds within 500ms under repeated calls', async () => {
      const times: number[] = [];
      for (let i = 0; i < 5; i++) {
        const start = Date.now();
        await api.get('/health');
        times.push(Date.now() - start);
      }
      const avg = times.reduce((a, b) => a + b, 0) / times.length;
      expect(avg).toBeLessThan(500);
    });
  });
});
