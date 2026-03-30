#!/usr/bin/env node
/**
 * Performance Test Runner
 *
 * Runs comprehensive performance tests including:
 * - Unit test performance suites (Jest)
 * - Load testing (Artillery)
 * - Stress testing (Artillery)
 * - Spike testing (Artillery)
 * - Smart contract performance (Cargo)
 *
 * Usage: npm run test:performance:comprehensive
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const REPORTS_DIR = path.join(__dirname, '..', 'reports', 'performance');
const API_DIR = path.join(__dirname, '..', 'api');

// Ensure reports directory exists
if (!fs.existsSync(REPORTS_DIR)) {
  fs.mkdirSync(REPORTS_DIR, { recursive: true });
}

console.log('🚀 Starting Comprehensive Performance Testing Suite\n');

// Test results
const results = {
  jest: null,
  artillery: {
    load: null,
    stress: null,
    spike: null,
  },
  contract: null,
  summary: null,
};

function runCommand(command, args = [], options = {}) {
  try {
    console.log(`📋 Running: ${command} ${args.join(' ')}`);
    const result = execSync(`${command} ${args.join(' ')}`, {
      stdio: 'inherit',
      cwd: options.cwd || process.cwd(),
      ...options,
    });
    console.log('✅ Success\n');
    return { success: true, output: result };
  } catch (error) {
    console.log('❌ Failed\n');
    return { success: false, error };
  }
}

async function runArtilleryTest(testFile, name) {
  const testPath = path.join(__dirname, '..', 'tests', 'load', testFile);

  if (!fs.existsSync(testPath)) {
    console.log(`⚠️  Artillery test file not found: ${testPath}`);
    return { success: false, error: 'Test file not found' };
  }

  try {
    console.log(`🔫 Running Artillery ${name} test...`);

    // Run artillery test and capture output
    const result = await new Promise((resolve) => {
      const artillery = spawn('npx', ['artillery', 'run', testPath], {
        stdio: ['inherit', 'pipe', 'pipe'],
        cwd: process.cwd(),
      });

      let stdout = '';
      let stderr = '';

      artillery.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      artillery.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      artillery.on('close', (code) => {
        resolve({
          success: code === 0,
          output: stdout,
          error: stderr,
          code,
        });
      });
    });

    if (result.success) {
      console.log('✅ Artillery test completed\n');
    } else {
      console.log('❌ Artillery test failed\n');
    }

    return result;
  } catch (error) {
    console.log('❌ Artillery test error\n');
    return { success: false, error };
  }
}

async function main() {
  console.log('📊 Running Jest Performance Tests...');
  results.jest = runCommand('npm', ['run', 'test:performance'], { cwd: API_DIR });

  console.log('🔫 Running Artillery Load Tests...');
  results.artillery.load = await runArtilleryTest('load-test.yml', 'Load');

  console.log('🔥 Running Artillery Stress Tests...');
  results.artillery.stress = await runArtilleryTest('stress-test.yml', 'Stress');

  console.log('⚡ Running Artillery Spike Tests...');
  results.artillery.spike = await runArtilleryTest('spike-test.yml', 'Spike');

  console.log('🔗 Running Smart Contract Performance Tests...');
  results.contract = runCommand('./scripts/stress-test.sh');

  // Generate summary report
  console.log('📈 Generating Performance Report...');

  const report = {
    timestamp: new Date().toISOString(),
    results: {
      jest: results.jest?.success ? 'PASSED' : 'FAILED',
      artillery: {
        load: results.artillery.load?.success ? 'PASSED' : 'FAILED',
        stress: results.artillery.stress?.success ? 'PASSED' : 'FAILED',
        spike: results.artillery.spike?.success ? 'PASSED' : 'FAILED',
      },
      contract: results.contract?.success ? 'PASSED' : 'FAILED',
    },
    summary: {
      total: 6,
      passed: [
        results.jest?.success,
        results.artillery.load?.success,
        results.artillery.stress?.success,
        results.artillery.spike?.success,
        results.contract?.success,
      ].filter(Boolean).length,
      failed: [
        !results.jest?.success,
        !results.artillery.load?.success,
        !results.artillery.stress?.success,
        !results.artillery.spike?.success,
        !results.contract?.success,
      ].filter(Boolean).length,
    },
  };

  const reportPath = path.join(REPORTS_DIR, `performance-report-${Date.now()}.json`);
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

  console.log(`\n📊 Performance Test Summary:`);
  console.log(`Jest Tests: ${report.results.jest}`);
  console.log(`Load Tests: ${report.results.artillery.load}`);
  console.log(`Stress Tests: ${report.results.artillery.stress}`);
  console.log(`Spike Tests: ${report.results.artillery.spike}`);
  console.log(`Contract Tests: ${report.results.contract}`);
  console.log(`\n📈 Overall: ${report.summary.passed}/${report.summary.total} tests passed`);

  console.log(`\n📄 Detailed report saved to: ${reportPath}`);

  if (report.summary.failed > 0) {
    console.log('\n❌ Some performance tests failed. Check the detailed report for issues.');
    process.exit(1);
  } else {
    console.log('\n✅ All performance tests passed!');
  }
}

main().catch((error) => {
  console.error('💥 Performance testing failed:', error);
  process.exit(1);
});