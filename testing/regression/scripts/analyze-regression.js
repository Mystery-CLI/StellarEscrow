#!/usr/bin/env node
/**
 * Regression Analysis Script
 *
 * Analyzes trends across multiple regression runs to identify:
 * - Flaky tests (pass/fail inconsistently)
 * - Slow tests (duration trending up)
 * - Frequently failing suites
 *
 * Usage: node scripts/analyze-regression.js
 */

const fs = require('fs');
const path = require('path');

const reportsDir = path.join(__dirname, '..', 'reports');
const historyPath = path.join(reportsDir, 'history.json');
const summaryPath = path.join(reportsDir, 'regression-summary.json');

if (!fs.existsSync(summaryPath)) {
  console.error('No regression-summary.json found. Run: npm run report first.');
  process.exit(1);
}

const current = JSON.parse(fs.readFileSync(summaryPath, 'utf-8'));

// Load or initialize history
let history = [];
if (fs.existsSync(historyPath)) {
  history = JSON.parse(fs.readFileSync(historyPath, 'utf-8'));
}

// Append current run
history.push({
  timestamp: current.generated_at,
  commit: current.commit,
  branch: current.branch,
  status: current.status,
  total: current.total,
  passed: current.passed,
  failed: current.failed,
  pass_rate: current.pass_rate,
  duration_ms: current.duration_ms,
  failed_tests: current.failed_tests.map(t => t.name),
});

// Keep last 50 runs
if (history.length > 50) history = history.slice(-50);
fs.writeFileSync(historyPath, JSON.stringify(history, null, 2));

// ── Analysis ─────────────────────────────────────────────────────────────────

const recentRuns = history.slice(-10);

// Flaky test detection: tests that failed in some runs but passed in others
const testFailCounts = {};
for (const run of recentRuns) {
  for (const t of (run.failed_tests || [])) {
    testFailCounts[t] = (testFailCounts[t] || 0) + 1;
  }
}

const flakyTests = Object.entries(testFailCounts)
  .filter(([, count]) => count > 0 && count < recentRuns.length)
  .sort((a, b) => b[1] - a[1])
  .map(([name, count]) => ({ name, fail_count: count, runs_analyzed: recentRuns.length }));

// Pass rate trend
const passRateTrend = recentRuns.map(r => r.pass_rate);
const avgPassRate = passRateTrend.reduce((a, b) => a + b, 0) / passRateTrend.length;
const trendDirection = passRateTrend.length >= 2
  ? passRateTrend[passRateTrend.length - 1] > passRateTrend[0] ? 'improving' : 'degrading'
  : 'stable';

// Consecutive failures
let consecutiveFailures = 0;
for (let i = history.length - 1; i >= 0; i--) {
  if (history[i].status === 'FAILED') consecutiveFailures++;
  else break;
}

const analysis = {
  generated_at: new Date().toISOString(),
  runs_analyzed: recentRuns.length,
  avg_pass_rate: parseFloat(avgPassRate.toFixed(1)),
  trend: trendDirection,
  consecutive_failures: consecutiveFailures,
  flaky_tests: flakyTests,
  current_run: {
    status: current.status,
    pass_rate: current.pass_rate,
    failed: current.failed,
  },
};

fs.writeFileSync(path.join(reportsDir, 'regression-analysis.json'), JSON.stringify(analysis, null, 2));

console.log('\n══════════════════════════════════════════');
console.log('  Regression Analysis');
console.log('══════════════════════════════════════════');
console.log(`  Runs analyzed:        ${analysis.runs_analyzed}`);
console.log(`  Avg pass rate:        ${analysis.avg_pass_rate}%`);
console.log(`  Trend:                ${analysis.trend}`);
console.log(`  Consecutive failures: ${analysis.consecutive_failures}`);
console.log(`  Flaky tests:          ${flakyTests.length}`);

if (flakyTests.length > 0) {
  console.log('\n  Flaky tests:');
  flakyTests.forEach(t => console.log(`    - ${t.name} (failed ${t.fail_count}/${t.runs_analyzed} runs)`));
}

if (consecutiveFailures >= 3) {
  console.warn(`\n  ⚠ WARNING: ${consecutiveFailures} consecutive failing runs detected!`);
}

console.log('\n══════════════════════════════════════════\n');
