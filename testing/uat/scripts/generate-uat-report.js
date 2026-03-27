#!/usr/bin/env node
/**
 * UAT Report Generator
 *
 * Produces an HTML stakeholder report from UAT results.
 * Groups results by Acceptance Criteria for easy sign-off.
 */

const fs = require('fs');
const path = require('path');

const reportsDir = path.join(__dirname, '..', 'reports');
const resultsPath = path.join(reportsDir, 'uat-results.json');

if (!fs.existsSync(resultsPath)) {
  console.error('No uat-results.json found. Run: npm run test:ci first.');
  process.exit(1);
}

const results = JSON.parse(fs.readFileSync(resultsPath, 'utf-8'));
const { summary, scenarios, meta } = results;

// Group by acceptance criteria
const byAC = {};
for (const s of scenarios) {
  const key = s.acceptance_criteria || 'General';
  if (!byAC[key]) byAC[key] = [];
  byAC[key].push(s);
}

const statusIcon = (status) => status === 'passed' ? '✅' : status === 'failed' ? '❌' : '⏭';
const statusColor = (status) => status === 'passed' ? '#22c55e' : status === 'failed' ? '#ef4444' : '#f59e0b';

const acSections = Object.entries(byAC).sort().map(([ac, tests]) => {
  const allPassed = tests.every(t => t.status === 'passed');
  const rows = tests.map(t => `
    <tr>
      <td>${statusIcon(t.status)}</td>
      <td>${escapeHtml(t.name)}</td>
      <td style="color:${statusColor(t.status)}">${t.status}</td>
      <td>${t.duration_ms}ms</td>
    </tr>`).join('');

  return `
  <div class="ac-section">
    <h3>${allPassed ? '✅' : '❌'} ${ac}</h3>
    <table>
      <thead><tr><th></th><th>Test</th><th>Status</th><th>Duration</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
  </div>`;
}).join('');

const overallStatus = summary.failed === 0 ? 'ACCEPTED' : 'NEEDS REVIEW';
const statusBg = summary.failed === 0 ? '#22c55e' : '#ef4444';

const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>StellarEscrow UAT Report</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 0; padding: 24px; background: #0f172a; color: #e2e8f0; }
    h1 { margin: 0 0 4px; font-size: 24px; }
    h2 { font-size: 18px; margin: 32px 0 12px; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; }
    h3 { font-size: 15px; margin: 0 0 8px; }
    .badge { display: inline-block; padding: 4px 12px; border-radius: 9999px; font-weight: 700; font-size: 14px; background: ${statusBg}; color: #fff; }
    .meta { color: #94a3b8; font-size: 13px; margin: 8px 0 24px; }
    .stats { display: flex; gap: 16px; margin-bottom: 24px; flex-wrap: wrap; }
    .stat { background: #1e293b; border-radius: 8px; padding: 16px 24px; min-width: 100px; text-align: center; }
    .stat-value { font-size: 28px; font-weight: 700; }
    .stat-label { font-size: 12px; color: #94a3b8; margin-top: 4px; }
    .ac-section { background: #1e293b; border-radius: 8px; padding: 16px; margin-bottom: 16px; }
    table { width: 100%; border-collapse: collapse; }
    th { background: #334155; padding: 8px 12px; text-align: left; font-size: 11px; text-transform: uppercase; color: #94a3b8; }
    td { padding: 8px 12px; border-top: 1px solid #334155; font-size: 13px; }
    .sign-off { background: #1e293b; border-radius: 8px; padding: 24px; margin-top: 32px; }
    .sign-off table td { border: 1px solid #334155; padding: 12px; min-width: 150px; }
  </style>
</head>
<body>
  <h1>StellarEscrow UAT Report <span class="badge">${overallStatus}</span></h1>
  <div class="meta">
    Environment: <strong>${meta.environment}</strong> &nbsp;|&nbsp;
    Base URL: <strong>${meta.base_url}</strong> &nbsp;|&nbsp;
    Commit: <strong>${meta.commit.slice(0, 8)}</strong> &nbsp;|&nbsp;
    ${new Date(meta.timestamp).toUTCString()}
  </div>

  <div class="stats">
    <div class="stat"><div class="stat-value">${summary.total}</div><div class="stat-label">Total</div></div>
    <div class="stat"><div class="stat-value" style="color:#22c55e">${summary.passed}</div><div class="stat-label">Passed</div></div>
    <div class="stat"><div class="stat-value" style="color:#ef4444">${summary.failed}</div><div class="stat-label">Failed</div></div>
    <div class="stat"><div class="stat-value">${summary.pass_rate}%</div><div class="stat-label">Pass Rate</div></div>
  </div>

  <h2>Acceptance Criteria Results</h2>
  ${acSections}

  <div class="sign-off">
    <h2 style="margin-top:0">Stakeholder Sign-Off</h2>
    <p style="color:#94a3b8;font-size:13px">Please review the results above and sign off on acceptance.</p>
    <table>
      <tr>
        <th>Role</th><th>Name</th><th>Signature</th><th>Date</th><th>Decision</th>
      </tr>
      <tr><td>Product Owner</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>☐ Accept &nbsp; ☐ Reject</td></tr>
      <tr><td>Compliance Officer</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>☐ Accept &nbsp; ☐ Reject</td></tr>
      <tr><td>DevOps Lead</td><td>&nbsp;</td><td>&nbsp;</td><td>&nbsp;</td><td>☐ Accept &nbsp; ☐ Reject</td></tr>
    </table>
  </div>
</body>
</html>`;

fs.writeFileSync(path.join(reportsDir, 'uat-report.html'), html);

const jsonSummary = {
  status: overallStatus,
  generated_at: new Date().toISOString(),
  environment: meta.environment,
  pass_rate: summary.pass_rate,
  total: summary.total,
  passed: summary.passed,
  failed: summary.failed,
  acceptance_criteria: Object.fromEntries(
    Object.entries(byAC).map(([ac, tests]) => [
      ac,
      { total: tests.length, passed: tests.filter(t => t.status === 'passed').length, status: tests.every(t => t.status === 'passed') ? 'accepted' : 'failed' }
    ])
  ),
};

fs.writeFileSync(path.join(reportsDir, 'uat-summary.json'), JSON.stringify(jsonSummary, null, 2));

console.log(`\n[UAT Report] Status: ${overallStatus}`);
console.log(`[UAT Report] ${summary.passed}/${summary.total} scenarios passed (${summary.pass_rate}%)`);
console.log(`[UAT Report] HTML: ${path.join(reportsDir, 'uat-report.html')}`);

process.exit(summary.failed > 0 ? 1 : 0);

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
