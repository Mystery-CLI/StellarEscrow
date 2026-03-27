'use strict';

const fs = require('fs');
const path = require('path');

class UatReporter {
  constructor(_globalConfig, options) {
    this._outputFile = (options || {}).outputFile || 'reports/uat-results.json';
  }

  onRunComplete(_contexts, results) {
    const scenarios = [];

    for (const suite of results.testResults) {
      for (const test of suite.testResults) {
        // Extract AC number from test name if present
        const acMatch = test.fullName.match(/AC-(\d+)/);
        scenarios.push({
          suite: path.basename(suite.testFilePath || ''),
          name: test.fullName,
          acceptance_criteria: acMatch ? `AC-${acMatch[1]}` : null,
          status: test.status,
          duration_ms: test.duration || 0,
          error: test.failureMessages?.join('\n') || null,
        });
      }
    }

    const output = {
      meta: {
        timestamp: new Date().toISOString(),
        environment: process.env.UAT_ENV || process.env.TEST_ENV || 'staging',
        base_url: process.env.UAT_BASE_URL || 'http://localhost:3000',
        commit: process.env.GITHUB_SHA || 'local',
      },
      summary: {
        total: results.numTotalTests,
        passed: results.numPassedTests,
        failed: results.numFailedTests,
        skipped: results.numPendingTests,
        pass_rate: results.numTotalTests > 0
          ? parseFloat(((results.numPassedTests / results.numTotalTests) * 100).toFixed(1))
          : 0,
      },
      scenarios,
    };

    const outputPath = path.isAbsolute(this._outputFile)
      ? this._outputFile
      : path.join(process.cwd(), this._outputFile);

    const dir = path.dirname(outputPath);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });

    fs.writeFileSync(outputPath, JSON.stringify(output, null, 2));
  }
}

module.exports = UatReporter;
