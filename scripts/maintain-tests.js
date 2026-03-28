#!/usr/bin/env node
/**
 * Flaky Test Maintenance Tool
 *
 * Reads regression-analysis.json history and flags tests that:
 *   - Failed in ≥ 30% of recent runs (flaky threshold)
 *   - Have been consistently failing for ≥ 3 consecutive runs (broken)
 *
 * Outputs a maintenance report and exits non-zero if broken tests exist.
 *
 * Usage: node scripts/maintain-tests.js [--threshold 0.3] [--window 10]
 */

'use strict';

const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
const threshold = parseFloat(args.find(a => a.startsWith('--threshold='))?.split('=')[1] ?? '0.3');
const window    = parseInt(args.find(a => a.startsWith('--window='))?.split('=')[1]    ?? '10', 10);

const reportsDir  = path.join(__dirname, '..', 'testing', 'regression', 'reports');
const historyPath = path.join(reportsDir, 'history.json');
const outputPath  = path.join(reportsDir, 'maintenance-report.json');

if (!fs.existsSync(historyPath)) {
  console.log('No history.json found — nothing to analyze yet.');
  process.exit(0);
}

const history = JSON.parse(fs.readFileSync(historyPath, 'utf-8'));
const recent  = history.slice(-window);

// Count failures per test across recent runs
const failCounts = {};
for (const run of recent) {
  for (const t of (run.failed_tests ?? [])) {
    failCounts[t] = (failCounts[t] ?? 0) + 1;
  }
}

const flaky  = [];
const broken = [];

for (const [name, count] of Object.entries(failCounts)) {
  const rate = count / recent.length;
  if (rate >= 1.0) {
    broken.push({ name, fail_count: count, fail_rate: rate });
  } else if (rate >= threshold) {
    flaky.push({ name, fail_count: count, fail_rate: parseFloat(rate.toFixed(2)) });
  }
}

const report = {
  generated_at: new Date().toISOString(),
  window_size:  recent.length,
  threshold,
  flaky_tests:  flaky.sort((a, b) => b.fail_rate - a.fail_rate),
  broken_tests: broken,
  recommendations: [
    ...broken.map(t => `REMOVE or FIX: "${t.name}" — failed every run in window`),
    ...flaky.map(t  => `INVESTIGATE: "${t.name}" — flaky (${(t.fail_rate * 100).toFixed(0)}% failure rate)`),
  ],
};

fs.mkdirSync(reportsDir, { recursive: true });
fs.writeFileSync(outputPath, JSON.stringify(report, null, 2));

console.log('\n══════════════════════════════════════════');
console.log('  Test Maintenance Report');
console.log('══════════════════════════════════════════');
console.log(`  Window:        last ${report.window_size} runs`);
console.log(`  Flaky tests:   ${flaky.length}`);
console.log(`  Broken tests:  ${broken.length}`);

if (report.recommendations.length > 0) {
  console.log('\n  Recommendations:');
  report.recommendations.forEach(r => console.log(`    • ${r}`));
}
console.log('══════════════════════════════════════════\n');

if (broken.length > 0) {
  console.error(`❌ ${broken.length} broken test(s) detected. Fix or remove before merging.`);
  process.exit(1);
}
