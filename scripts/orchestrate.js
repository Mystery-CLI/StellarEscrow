#!/usr/bin/env node
/**
 * Test Orchestrator — runs all test suites in dependency order,
 * collects results, and exits with a non-zero code on any failure.
 *
 * Usage:
 *   node scripts/orchestrate.js [--suite unit|api|uat|a11y|all] [--env staging]
 */

'use strict';

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
const suiteArg = args.find(a => a.startsWith('--suite='))?.split('=')[1] ?? 'all';
const env = args.find(a => a.startsWith('--env='))?.split('=')[1] ?? 'development';

const SUITES = {
  unit:       { cwd: 'testing/regression',     cmd: 'npm run test:ci', label: 'Unit Regression', needsApi: true },
  api:        { cwd: 'api',                     cmd: 'npm run test',    label: 'API Tests' },
  uat:        { cwd: 'testing/uat',            cmd: 'npm run test:ci', label: 'UAT' },
  a11y:       { cwd: 'testing/accessibility',  cmd: 'npm run test:ci', label: 'Accessibility' },
};

const ORDER = ['unit', 'api', 'uat', 'a11y'];
const toRun = suiteArg === 'all' ? ORDER : [suiteArg];

const reportsDir = path.join(__dirname, '..', 'reports');
fs.mkdirSync(reportsDir, { recursive: true });

function checkApiHealth(apiBaseURL = process.env.API_BASE_URL || 'http://localhost:3000') {
  return new Promise((resolve) => {
    const url = new URL('/health', apiBaseURL).toString();
    const client = url.startsWith('https') ? require('https') : require('http');
    const req = client.get(url, { timeout: 4000 }, (res) => {
      res.resume();
      resolve(res.statusCode === 200);
    });
    req.on('error', () => resolve(false));
    req.on('timeout', () => { req.destroy(); resolve(false); });
  });
}

(async () => {
  const results = [];
  let anyFailed = false;

  for (const key of toRun) {
    const suite = SUITES[key];
    if (!suite) { console.error(`Unknown suite: ${key}`); process.exit(1); }

    console.log(`\n▶ Running ${suite.label}...`);

    if (suite.needsApi) {
      const apiHealthy = await checkApiHealth(process.env.API_BASE_URL || 'http://localhost:3000');
      if (!apiHealthy) {
        console.error(`❌ Skipping ${suite.label}: API health check failed at ${process.env.API_BASE_URL || 'http://localhost:3000'}`);
        results.push({ suite: key, label: suite.label, status: 'skipped', duration_ms: 0 });
        continue;
      }
    }

    const start = Date.now();
    let status = 'passed';

    try {
      execSync(suite.cmd, {
        cwd: path.join(__dirname, '..', '..', suite.cwd),
        stdio: 'inherit',
        env: { ...process.env, TEST_ENV: env },
      });
    } catch {
      status = 'failed';
      anyFailed = true;
    }

    results.push({ suite: key, label: suite.label, status, duration_ms: Date.now() - start });
    console.log(`${status === 'passed' ? '✅' : '❌'} ${suite.label} — ${status}`);
  }

  // Write orchestration summary
  const summary = {
    generated_at: new Date().toISOString(),
    environment: env,
    commit: process.env.GITHUB_SHA ?? 'local',
    branch: process.env.GITHUB_REF_NAME ?? 'local',
    suites: results,
    overall: anyFailed ? 'FAILED' : 'PASSED',
  };

  fs.writeFileSync(path.join(reportsDir, 'orchestration-summary.json'), JSON.stringify(summary, null, 2));

  console.log(`\n${'═'.repeat(50)}`);
  console.log(`  Overall: ${summary.overall}`);
  results.forEach(r => console.log(`  ${r.status === 'passed' ? '✅' : '❌'} ${r.label} (${r.duration_ms}ms)`));
  console.log(`${'═'.repeat(50)}\n`);

  process.exit(anyFailed ? 1 : 0);
})();
