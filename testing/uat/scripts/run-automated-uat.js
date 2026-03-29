#!/usr/bin/env node
/**
 * Automated UAT Runner
 *
 * Orchestrates the full UAT pipeline:
 *   1. Run UAT scenarios
 *   2. Generate HTML report
 *   3. Collect any pending feedback
 *   4. Post summary to webhook (Slack / Teams) if configured
 *
 * Usage: node scripts/run-automated-uat.js [--env staging] [--webhook <url>]
 */

const { execSync } = require('child_process');
const fs   = require('fs');
const path = require('path');
const http = require('https');

const args = process.argv.slice(2);
const getArg = (flag) => { const i = args.indexOf(flag); return i !== -1 ? args[i + 1] : null; };

const env        = getArg('--env')     || process.env.UAT_ENV     || 'staging';
const webhookUrl = getArg('--webhook') || process.env.UAT_WEBHOOK || '';
const reportsDir = path.join(__dirname, '..', 'reports');

if (!fs.existsSync(reportsDir)) fs.mkdirSync(reportsDir, { recursive: true });

console.log(`\n[UAT] Starting automated UAT run — environment: ${env}\n`);

let exitCode = 0;

// Step 1: Run scenarios
try {
  execSync('npm run test:ci', {
    cwd: path.join(__dirname, '..'),
    stdio: 'inherit',
    env: { ...process.env, UAT_ENV: env },
  });
} catch (e) {
  exitCode = 1;
  console.warn('[UAT] Some scenarios failed — continuing to generate report.');
}

// Step 2: Generate report
try {
  execSync('node scripts/generate-uat-report.js', {
    cwd: path.join(__dirname, '..'),
    stdio: 'inherit',
  });
} catch (e) {
  console.warn('[UAT] Report generation failed:', e.message);
}

// Step 3: Collect feedback
try {
  execSync('node scripts/collect-feedback.js', {
    cwd: path.join(__dirname, '..'),
    stdio: 'inherit',
  });
} catch (e) {
  // Feedback is optional
}

// Step 4: Post to webhook
const summaryPath = path.join(reportsDir, 'uat-summary.json');
if (webhookUrl && fs.existsSync(summaryPath)) {
  const summary = JSON.parse(fs.readFileSync(summaryPath, 'utf-8'));
  const icon = summary.status === 'ACCEPTED' ? '✅' : '❌';
  const payload = JSON.stringify({
    text: `${icon} *StellarEscrow UAT — ${summary.status}*\nEnvironment: \`${env}\` | Pass rate: ${summary.pass_rate}% (${summary.passed}/${summary.total})\n${summary.failed > 0 ? `⚠ ${summary.failed} scenario(s) failed` : 'All scenarios passed.'}`,
  });

  const url = new URL(webhookUrl);
  const req = http.request({
    hostname: url.hostname,
    path: url.pathname + url.search,
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(payload) },
  });
  req.write(payload);
  req.end();
  console.log('[UAT] Webhook notification sent.');
}

console.log(`\n[UAT] Automated run complete. Exit code: ${exitCode}`);
process.exit(exitCode);
