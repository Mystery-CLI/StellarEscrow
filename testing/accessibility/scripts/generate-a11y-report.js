#!/usr/bin/env node
/**
 * Accessibility Report Generator
 * Produces an HTML report from a11y test results, grouped by WCAG criterion.
 */

const fs = require('fs');
const path = require('path');

const reportsDir = path.join(__dirname, '..', 'reports');
const resultsPath = path.join(reportsDir, 'a11y-results.json');

if (!fs.existsSync(resultsPath)) {
  console.error('No a11y-results.json found. Run: npm run test:ci first.');
  process.exit(1);
}

const { summary, tests, meta } = JSON.parse(fs.readFileSync(resultsPath, 'utf-8'));

// Group by WCAG criterion
const byCriterion = {};
for (const t of tests) {
  const key = t.wcag_criterion || 'General';
  if (!byCriterion[key]) byCriterion[key] = [];
  byCriterion[key].push(t);
}

const overallStatus = summary.failed === 0 ? 'PASSED' : 'FAILED';
const statusBg = summary.failed === 0 ? '#22c55e' : '#ef4444';

const criterionSections = Object.entries(byCriterion).sort().map(([criterion, tests]) => {
  const allPassed = tests.every(t => t.status === 'passed');
  const rows = tests.map(t => `
    <tr>
      <td>${t.status === 'passed' ? '✅' : '❌'}</td>
      <td>${escapeHtml(t.name)}</td>
      <td style="color:${t.status === 'passed' ? '#22c55e' : '#ef4444'}">${t.status}</td>
      <td>${t.duration_ms}ms</td>
    </tr>`).join('');

  return `
  <div class="criterion">
    <h3>${allPassed ? '✅' : '❌'} WCAG ${criterion}</h3>
    <table>
      <thead><tr><th></th><th>Test</th><th>Status</th><th>Duration</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
  </div>`;
}).join('');

const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>StellarEscrow Accessibility Report</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 0; padding: 24px; background: #0f172a; color: #e2e8f0; }
    h1 { margin: 0 0 4px; font-size: 24px; }
    h2 { font-size: 16px; margin: 32px 0 12px; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; }
    h3 { font-size: 14px; margin: 0 0 8px; }
    .badge { display: inline-block; padding: 4px 12px; border-radius: 9999px; font-weight: 700; font-size: 14px; background: ${statusBg}; color: #fff; }
    .meta { color: #94a3b8; font-size: 13px; margin: 8px 0 24px; }
    .stats { display: flex; gap: 16px; margin-bottom: 24px; }
    .stat { background: #1e293b; border-radius: 8px; padding: 16px 24px; text-align: center; }
    .stat-value { font-size: 28px; font-weight: 700; }
    .stat-label { font-size: 12px; color: #94a3b8; margin-top: 4px; }
    .criterion { background: #1e293b; border-radius: 8px; padding: 16px; margin-bottom: 12px; }
    table { width: 100%; border-collapse: collapse; }
    th { background: #334155; padding: 8px 12px; text-align: left; font-size: 11px; text-transform: uppercase; color: #94a3b8; }
    td { padding: 8px 12px; border-top: 1px solid #334155; font-size: 13px; }
  </style>
</head>
<body>
  <h1>StellarEscrow Accessibility Report <span class="badge">${overallStatus}</span></h1>
  <div class="meta">
    WCAG Level: <strong>AA</strong> &nbsp;|&nbsp;
    Commit: <strong>${meta.commit.slice(0, 8)}</strong> &nbsp;|&nbsp;
    ${new Date(meta.timestamp).toUTCString()}
  </div>

  <div class="stats">
    <div class="stat"><div class="stat-value">${summary.total}</div><div class="stat-label">Total</div></div>
    <div class="stat"><div class="stat-value" style="color:#22c55e">${summary.passed}</div><div class="stat-label">Passed</div></div>
    <div class="stat"><div class="stat-value" style="color:#ef4444">${summary.failed}</div><div class="stat-label">Failed</div></div>
    <div class="stat"><div class="stat-value">${summary.pass_rate}%</div><div class="stat-label">Pass Rate</div></div>
  </div>

  <h2>Results by WCAG Criterion</h2>
  ${criterionSections}
</body>
</html>`;

fs.writeFileSync(path.join(reportsDir, 'a11y-report.html'), html);
console.log(`[a11y-report] ${overallStatus} — ${summary.passed}/${summary.total} (${summary.pass_rate}%)`);
console.log(`[a11y-report] HTML: ${path.join(reportsDir, 'a11y-report.html')}`);
process.exit(summary.failed > 0 ? 1 : 0);

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
