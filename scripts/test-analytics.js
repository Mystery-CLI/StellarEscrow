#!/usr/bin/env node
/**
 * Test Analytics — aggregates results across all suites into a single
 * analytics report used by the Grafana dashboard and CI summary.
 *
 * Reads:  reports/orchestration-summary.json
 *         testing/regression/reports/regression-analysis.json  (if present)
 *         testing/uat/reports/uat-results.json                 (if present)
 *         testing/accessibility/reports/a11y-results.json      (if present)
 *
 * Writes: reports/test-analytics.json
 */

'use strict';

const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const reportsDir = path.join(root, 'reports');

function readJson(p) {
  return fs.existsSync(p) ? JSON.parse(fs.readFileSync(p, 'utf-8')) : null;
}

const orchestration = readJson(path.join(reportsDir, 'orchestration-summary.json'));
const regression    = readJson(path.join(root, 'testing/regression/reports/regression-analysis.json'));
const uat           = readJson(path.join(root, 'testing/uat/reports/uat-results.json'));
const a11y          = readJson(path.join(root, 'testing/accessibility/reports/a11y-results.json'));

// ── Aggregate totals ──────────────────────────────────────────────────────────
const suites = orchestration?.suites ?? [];
const totalDuration = suites.reduce((s, r) => s + (r.duration_ms ?? 0), 0);
const passedSuites  = suites.filter(r => r.status === 'passed').length;

const analytics = {
  generated_at: new Date().toISOString(),
  commit:   orchestration?.commit  ?? 'unknown',
  branch:   orchestration?.branch  ?? 'unknown',
  overall:  orchestration?.overall ?? 'UNKNOWN',

  suites: {
    total:  suites.length,
    passed: passedSuites,
    failed: suites.length - passedSuites,
    duration_ms: totalDuration,
    breakdown: suites,
  },

  regression: regression ? {
    avg_pass_rate:        regression.avg_pass_rate,
    trend:                regression.trend,
    consecutive_failures: regression.consecutive_failures,
    flaky_test_count:     regression.flaky_tests?.length ?? 0,
    flaky_tests:          regression.flaky_tests ?? [],
  } : null,

  uat: uat ? {
    total:     uat.summary?.total    ?? 0,
    passed:    uat.summary?.passed   ?? 0,
    failed:    uat.summary?.failed   ?? 0,
    pass_rate: uat.summary?.pass_rate ?? 0,
  } : null,

  accessibility: a11y ? {
    violations: a11y.summary?.violations ?? 0,
    warnings:   a11y.summary?.warnings   ?? 0,
    pass_rate:  a11y.summary?.pass_rate  ?? 0,
  } : null,
};

fs.mkdirSync(reportsDir, { recursive: true });
fs.writeFileSync(path.join(reportsDir, 'test-analytics.json'), JSON.stringify(analytics, null, 2));

console.log('\n══════════════════════════════════════════');
console.log('  Test Analytics');
console.log('══════════════════════════════════════════');
console.log(`  Overall:          ${analytics.overall}`);
console.log(`  Suites passed:    ${analytics.suites.passed}/${analytics.suites.total}`);
console.log(`  Total duration:   ${(totalDuration / 1000).toFixed(1)}s`);
if (analytics.regression) {
  console.log(`  Regression trend: ${analytics.regression.trend} (${analytics.regression.avg_pass_rate}%)`);
  console.log(`  Flaky tests:      ${analytics.regression.flaky_test_count}`);
}
if (analytics.uat)
  console.log(`  UAT pass rate:    ${analytics.uat.pass_rate}%`);
if (analytics.accessibility)
  console.log(`  A11y violations:  ${analytics.accessibility.violations}`);
console.log('══════════════════════════════════════════\n');
