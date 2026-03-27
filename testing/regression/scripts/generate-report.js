#!/usr/bin/env node
/**
 * Regression Report Generator
 *
 * Reads Jest JSON output and produces a human-readable HTML + JSON report.
 * Usage: node scripts/generate-report.js
 */

const fs = require('fs');
const path = require('path');

const reportsDir = path.join(__dirname, '..', 'reports');
const resultsPath = path.join(reportsDir, 'regression-results.json');
const metaPath    = path.join(reportsDir, 'run-meta.json');

if (!fs.existsSync(resultsPath)) {
  console.error('No regression-results.json found. Run: npm run test:ci first.');
  process.exit(1);
}

const results = JSON.parse(fs.readFileSync(resultsPath, 'utf-8'));
const meta    = fs.existsSync(metaPath) ? JSON.parse(fs.readFileSync(metaPath, 'utf-8')) : {};

// ── Aggregate stats ──────────────────────────────────────────────────────────

let totalTests = 0, passed = 0, failed = 0, skipped = 0;
const failedTests = [];
const suiteStats = [];

for (const suite of (results.testResults || [])) {
  let suitePassed = 0, suiteFailed = 0;
  for (const test of (suite.testResults || [])) {
    totalTests++;
    if (test.status === 'passed')  { passed++;  suitePassed++; }
    if (test.status === 'failed')  { failed++;  suiteFailed++; failedTests.push({ suite: suite.testFilePath, name: test.fullName, error: (test.failureMessages || []).join('\n') }); }
    if (test.status === 'pending') { skipped++; }
  }
  suiteStats.push({
    file: path.relative(process.cwd(), suite.testFilePath || ''),
    passed: suitePassed,
    failed: suiteFailed,
    duration_ms: suite.perfStats?.runtime || 0,
  });
}

const passRate = totalTests > 0 ? ((passed / totalTests) * 100).toFixed(1) : '0.0';
const status = failed === 0 ? 'PASSED' : 'FAILED';

// ── JSON summary ─────────────────────────────────────────────────────────────

const summary = {
  status,
  generated_at: new Date().toISOString(),
  environment: meta.environment || 'unknown',
  commit: meta.commit || 'unknown',
  branch: meta.branch || 'unknown',
  duration_ms: meta.durationMs || 0,
  total: totalTests,
  passed,
  failed,
  skipped,
  pass_rate: parseFloat(passRate),
  suites: suiteStats,
  failed_tests: failedTests,
};

fs.writeFileSync(path.join(reportsDir, 'regression-summary.json'), JSON.stringify(summary, null, 2));

// ── HTML report ───────────────────────────────────────────────────────────────

const statusColor = status === 'PASSED' ? '#22c55e' : '#ef4444';
const suiteRows = suiteStats.map(s => `
  <tr>
    <td>${s.file}</td>
    <td style="color:#22c55e">${s.passed}</td>
    <td style="color:${s.failed > 0 ? '#ef4444' : '#6b7280'}">${s.failed}</td>
    <td>${s.duration_ms}ms</td>
  </tr>`).join('');

const failedRows = failedTests.map(f => `
  <tr>
    <td>${f.suite}</td>
    <td>${f.name}</td>
    <td><pre style="font-size:11px;white-space:pre-wrap">${escapeHtml(f.error.slice(0, 300))}</pre></td>
  </tr>`).join('');

const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>StellarEscrow Regression Report</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 0; padding: 24px; background: #0f172a; color: #e2e8f0; }
    h1 { margin: 0 0 4px; font-size: 24px; }
    .badge { display: inline-block; padding: 4px 12px; border-radius: 9999px; font-weight: 700; font-size: 14px; background: ${statusColor}; color: #fff; }
    .meta { color: #94a3b8; font-size: 13px; margin: 8px 0 24px; }
    .stats { display: flex; gap: 16px; margin-bottom: 24px; }
    .stat { background: #1e293b; border-radius: 8px; padding: 16px 24px; min-width: 100px; text-align: center; }
    .stat-value { font-size: 28px; font-weight: 700; }
    .stat-label { font-size: 12px; color: #94a3b8; margin-top: 4px; }
    table { width: 100%; border-collapse: collapse; background: #1e293b; border-radius: 8px; overflow: hidden; margin-bottom: 24px; }
    th { background: #334155; padding: 10px 14px; text-align: left; font-size: 12px; text-transform: uppercase; color: #94a3b8; }
    td { padding: 10px 14px; border-top: 1px solid #334155; font-size: 13px; }
    h2 { font-size: 16px; margin: 24px 0 8px; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; }
  </style>
</head>
<body>
  <h1>StellarEscrow Regression Report <span class="badge">${status}</span></h1>
  <div class="meta">
    Branch: <strong>${summary.branch}</strong> &nbsp;|&nbsp;
    Commit: <strong>${summary.commit.slice(0, 8)}</strong> &nbsp;|&nbsp;
    Env: <strong>${summary.environment}</strong> &nbsp;|&nbsp;
    ${new Date(summary.generated_at).toUTCString()}
  </div>

  <div class="stats">
    <div class="stat"><div class="stat-value">${totalTests}</div><div class="stat-label">Total</div></div>
    <div class="stat"><div class="stat-value" style="color:#22c55e">${passed}</div><div class="stat-label">Passed</div></div>
    <div class="stat"><div class="stat-value" style="color:#ef4444">${failed}</div><div class="stat-label">Failed</div></div>
    <div class="stat"><div class="stat-value" style="color:#f59e0b">${skipped}</div><div class="stat-label">Skipped</div></div>
    <div class="stat"><div class="stat-value">${passRate}%</div><div class="stat-label">Pass Rate</div></div>
  </div>

  <h2>Test Suites</h2>
  <table>
    <thead><tr><th>Suite</th><th>Passed</th><th>Failed</th><th>Duration</th></tr></thead>
    <tbody>${suiteRows}</tbody>
  </table>

  ${failed > 0 ? `
  <h2>Failed Tests</h2>
  <table>
    <thead><tr><th>Suite</th><th>Test</th><th>Error</th></tr></thead>
    <tbody>${failedRows}</tbody>
  </table>` : '<p style="color:#22c55e">✓ All tests passed.</p>'}
</body>
</html>`;

fs.writeFileSync(path.join(reportsDir, 'regression-report.html'), html);

console.log(`\n[report] Status: ${status}`);
console.log(`[report] ${passed}/${totalTests} tests passed (${passRate}%)`);
console.log(`[report] HTML: ${path.join(reportsDir, 'regression-report.html')}`);
console.log(`[report] JSON: ${path.join(reportsDir, 'regression-summary.json')}`);

process.exit(failed > 0 ? 1 : 0);

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
