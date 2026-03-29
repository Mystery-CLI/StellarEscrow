#!/usr/bin/env node
/**
 * Change Detection Script
 *
 * Compares the current regression run results against a stored baseline.
 * Exits with code 1 if regressions are detected, 0 if clean.
 *
 * Usage:
 *   node scripts/detect-changes.js [--baseline reports/baseline.json] [--current reports/regression-results.json]
 */

const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
const getArg = (flag) => {
  const i = args.indexOf(flag);
  return i !== -1 ? args[i + 1] : null;
};

const baselinePath = getArg('--baseline') || path.join(__dirname, '..', 'reports', 'baseline.json');
const currentPath  = getArg('--current')  || path.join(__dirname, '..', 'reports', 'regression-results.json');

if (!fs.existsSync(currentPath)) {
  console.error(`[detect-changes] Current results not found: ${currentPath}`);
  console.error('Run the regression suite first: npm run test:ci');
  process.exit(1);
}

const current = JSON.parse(fs.readFileSync(currentPath, 'utf-8'));

// If no baseline exists, save current as baseline and exit clean
if (!fs.existsSync(baselinePath)) {
  console.log('[detect-changes] No baseline found. Saving current results as baseline.');
  fs.writeFileSync(baselinePath, JSON.stringify(current, null, 2));
  console.log(`[detect-changes] Baseline saved to ${baselinePath}`);
  process.exit(0);
}

const baseline = JSON.parse(fs.readFileSync(baselinePath, 'utf-8'));

// ── Compare test results ────────────────────────────────────────────────────

const regressions = [];
const improvements = [];

const baselineTests = indexTests(baseline);
const currentTests  = indexTests(current);

for (const [key, curr] of Object.entries(currentTests)) {
  const base = baselineTests[key];

  if (!base) {
    // New test — not a regression
    continue;
  }

  if (base.status === 'passed' && curr.status === 'failed') {
    regressions.push({
      test: key,
      baseline_status: base.status,
      current_status: curr.status,
      error: curr.failureMessages?.join('\n') || 'unknown',
    });
  }

  if (base.status === 'failed' && curr.status === 'passed') {
    improvements.push({ test: key });
  }
}

// ── Check for missing tests (deleted tests = potential coverage regression) ─

const deletedTests = Object.keys(baselineTests).filter(k => !currentTests[k]);
if (deletedTests.length > 0) {
  console.warn(`[detect-changes] WARNING: ${deletedTests.length} test(s) removed since baseline:`);
  deletedTests.forEach(t => console.warn(`  - ${t}`));
}

// ── Report ──────────────────────────────────────────────────────────────────

const report = {
  generated_at: new Date().toISOString(),
  baseline_commit: baseline.meta?.commit || 'unknown',
  current_commit: current.meta?.commit || process.env.GITHUB_SHA || 'local',
  regressions_count: regressions.length,
  improvements_count: improvements.length,
  deleted_tests_count: deletedTests.length,
  regressions,
  improvements,
  deleted_tests: deletedTests,
};

const reportPath = path.join(__dirname, '..', 'reports', 'change-detection.json');
fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

console.log('\n══════════════════════════════════════════');
console.log('  Regression Change Detection Report');
console.log('══════════════════════════════════════════');
console.log(`  Regressions:  ${regressions.length}`);
console.log(`  Improvements: ${improvements.length}`);
console.log(`  Deleted tests: ${deletedTests.length}`);
console.log(`  Report: ${reportPath}`);
console.log('══════════════════════════════════════════\n');

if (regressions.length > 0) {
  console.error('[detect-changes] REGRESSIONS DETECTED:');
  regressions.forEach(r => {
    console.error(`  ✗ ${r.test}`);
    console.error(`    ${r.error.split('\n')[0]}`);
  });
  process.exit(1);
} else {
  console.log('[detect-changes] ✓ No regressions detected.');
  process.exit(0);
}

// ── Helpers ─────────────────────────────────────────────────────────────────

function indexTests(results) {
  const index = {};
  const testResults = results.testResults || results.tests || [];
  for (const suite of testResults) {
    const suiteName = suite.testFilePath || suite.file || '';
    for (const test of (suite.testResults || suite.tests || [])) {
      const key = `${suiteName}::${test.fullName || test.name}`;
      index[key] = {
        status: test.status,
        failureMessages: test.failureMessages || [],
        duration: test.duration,
      };
    }
  }
  return index;
}
