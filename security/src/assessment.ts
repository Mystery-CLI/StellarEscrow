import { SecurityMonitor } from './monitoring';
import {
  complianceTestScenarios,
  defaultSecurityScenarios,
  penetrationTestScenarios,
  runSecurityScenarios,
  vulnerabilityScanScenarios,
} from './scenarios';
import {
  SecurityAssessmentType,
  SecurityFinding,
  SecurityReport,
  SecurityReportSummary,
  SecurityTestTarget,
} from './types';

const buildSummary = (findings: SecurityFinding[]): SecurityReportSummary => ({
  total: findings.length,
  passed: findings.filter((finding) => finding.passed).length,
  failed: findings.filter((finding) => !finding.passed).length,
  low: findings.filter((finding) => !finding.passed && finding.severity === 'low').length,
  medium: findings.filter((finding) => !finding.passed && finding.severity === 'medium').length,
  high: findings.filter((finding) => !finding.passed && finding.severity === 'high').length,
  critical: findings.filter((finding) => !finding.passed && finding.severity === 'critical').length,
});

const buildReport = (type: SecurityAssessmentType, findings: SecurityFinding[]): SecurityReport => {
  const summary = buildSummary(findings);
  return {
    type,
    findings,
    summary,
    passed: summary.failed === 0,
  };
};

export const runPenetrationTests = (target: SecurityTestTarget, monitor?: SecurityMonitor): SecurityReport =>
  buildReport('penetration', runSecurityScenarios(target, penetrationTestScenarios, monitor));

export const runVulnerabilityScan = (target: SecurityTestTarget, monitor?: SecurityMonitor): SecurityReport =>
  buildReport('vulnerability', runSecurityScenarios(target, vulnerabilityScanScenarios, monitor));

export const runComplianceChecks = (target: SecurityTestTarget, monitor?: SecurityMonitor): SecurityReport =>
  buildReport('compliance', runSecurityScenarios(target, complianceTestScenarios, monitor));

export const runSecurityScenarioSuite = (target: SecurityTestTarget, monitor?: SecurityMonitor): SecurityFinding[] =>
  runSecurityScenarios(target, defaultSecurityScenarios, monitor);

export interface SecurityAssessmentResult {
  generatedAt: number;
  overallPassed: boolean;
  reports: SecurityReport[];
  findings: SecurityFinding[];
  totals: SecurityReportSummary;
  monitoring: ReturnType<SecurityMonitor['getSummary']>;
}

export const runSecurityAssessment = (
  target: SecurityTestTarget,
  monitor = new SecurityMonitor(target.monitoring?.endpoint ? { endpoint: target.monitoring.endpoint } : {})
): SecurityAssessmentResult => {
  const reports = [
    runPenetrationTests(target, monitor),
    runVulnerabilityScan(target, monitor),
    runComplianceChecks(target, monitor),
  ];
  const findings = reports.flatMap((report) => report.findings);
  const totals = buildSummary(findings);

  return {
    generatedAt: Date.now(),
    overallPassed: reports.every((report) => report.passed),
    reports,
    findings,
    totals,
    monitoring: monitor.getSummary(),
  };
};
