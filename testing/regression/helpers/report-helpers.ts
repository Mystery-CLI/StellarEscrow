interface CheckFixture {
  status: string;
  risk_score: number;
  aml_risk: number;
  kyc_status: string;
  jurisdiction?: string;
}

interface ReportSummary {
  total_checks: number;
  approved: number;
  rejected: number;
  blocked: number;
  requires_review: number;
  pending: number;
  avg_risk_score: number;
  high_risk_count: number;
}

export function build_report_from_fixtures(fixtures: CheckFixture[]): ReportSummary {
  let approved = 0, rejected = 0, blocked = 0, requires_review = 0, pending = 0;
  let risk_sum = 0, high_risk = 0;

  for (const f of fixtures) {
    switch (f.status) {
      case 'approved': approved++; break;
      case 'rejected': rejected++; break;
      case 'blocked': blocked++; break;
      case 'requires_review': requires_review++; break;
      default: pending++;
    }
    risk_sum += f.risk_score;
    if (f.risk_score >= 70) high_risk++;
  }

  return {
    total_checks: fixtures.length,
    approved,
    rejected,
    blocked,
    requires_review,
    pending,
    avg_risk_score: fixtures.length > 0 ? risk_sum / fixtures.length : 0,
    high_risk_count: high_risk,
  };
}
