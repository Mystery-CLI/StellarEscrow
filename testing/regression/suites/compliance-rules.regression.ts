/**
 * Compliance Rules Regression Suite
 *
 * Verifies that KYC/AML compliance logic hasn't regressed.
 * Tests the workflow state machine and reporting aggregation.
 */

import { is_valid_transition } from '../helpers/compliance-helpers';
import { build_report_from_fixtures } from '../helpers/report-helpers';

describe('Compliance Regression — Workflow state machine', () => {
  const allowed: [string, string][] = [
    ['pending', 'approved'],
    ['pending', 'rejected'],
    ['pending', 'requires_review'],
    ['pending', 'blocked'],
    ['requires_review', 'approved'],
    ['requires_review', 'rejected'],
    ['requires_review', 'blocked'],
  ];

  const forbidden: [string, string][] = [
    ['approved', 'pending'],
    ['rejected', 'approved'],
    ['blocked', 'approved'],
    ['approved', 'blocked'],
  ];

  allowed.forEach(([from, to]) => {
    it(`allows transition: ${from} → ${to}`, () => {
      expect(is_valid_transition(from, to)).toBe(true);
    });
  });

  forbidden.forEach(([from, to]) => {
    it(`forbids transition: ${from} → ${to}`, () => {
      expect(is_valid_transition(from, to)).toBe(false);
    });
  });
});

describe('Compliance Regression — Risk score calculation', () => {
  it('blocked address always has risk_score >= aml_threshold', () => {
    const check = build_report_from_fixtures([
      { status: 'blocked', risk_score: 85, aml_risk: 85, kyc_status: 'verified' },
    ]);
    expect(check.blocked).toBe(1);
    expect(check.avg_risk_score).toBeGreaterThanOrEqual(70);
  });

  it('approved address has risk_score < threshold', () => {
    const check = build_report_from_fixtures([
      { status: 'approved', risk_score: 10, aml_risk: 10, kyc_status: 'verified' },
    ]);
    expect(check.approved).toBe(1);
    expect(check.avg_risk_score).toBeLessThan(70);
  });
});

describe('Compliance Regression — Report aggregation', () => {
  it('correctly counts statuses', () => {
    const report = build_report_from_fixtures([
      { status: 'approved', risk_score: 5, aml_risk: 5, kyc_status: 'verified' },
      { status: 'approved', risk_score: 10, aml_risk: 10, kyc_status: 'verified' },
      { status: 'blocked', risk_score: 90, aml_risk: 90, kyc_status: 'rejected' },
      { status: 'requires_review', risk_score: 65, aml_risk: 65, kyc_status: 'pending' },
      { status: 'pending', risk_score: 20, aml_risk: 20, kyc_status: 'pending' },
    ]);

    expect(report.total_checks).toBe(5);
    expect(report.approved).toBe(2);
    expect(report.blocked).toBe(1);
    expect(report.requires_review).toBe(1);
    expect(report.pending).toBe(1);
  });

  it('calculates average risk score correctly', () => {
    const report = build_report_from_fixtures([
      { status: 'approved', risk_score: 10, aml_risk: 10, kyc_status: 'verified' },
      { status: 'approved', risk_score: 30, aml_risk: 30, kyc_status: 'verified' },
    ]);
    expect(report.avg_risk_score).toBeCloseTo(20, 1);
  });

  it('counts high_risk_count for scores >= 70', () => {
    const report = build_report_from_fixtures([
      { status: 'blocked', risk_score: 80, aml_risk: 80, kyc_status: 'rejected' },
      { status: 'blocked', risk_score: 95, aml_risk: 95, kyc_status: 'rejected' },
      { status: 'approved', risk_score: 20, aml_risk: 20, kyc_status: 'verified' },
    ]);
    expect(report.high_risk_count).toBe(2);
  });
});
