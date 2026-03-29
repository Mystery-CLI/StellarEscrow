'use strict';
/**
 * Custom Jest reporter that writes structured JSON output
 * compatible with the change detection and analysis scripts.
 */

const fs = require('fs');
const path = require('path');

class RegressionReporter {
  constructor(globalConfig, options) {
    this._options = options || {};
    this._outputFile = options.outputFile || 'reports/regression-results.json';
  }

  onRunComplete(_contexts, results) {
    const output = {
      meta: {
        commit: process.env.GITHUB_SHA || 'local',
        branch: process.env.GITHUB_REF_NAME || 'local',
        environment: process.env.TEST_ENV || 'development',
        timestamp: new Date().toISOString(),
      },
      numTotalTests: results.numTotalTests,
      numPassedTests: results.numPassedTests,
      numFailedTests: results.numFailedTests,
      numPendingTests: results.numPendingTests,
      success: results.success,
      testResults: results.testResults.map(suite => ({
        testFilePath: suite.testFilePath,
        status: suite.status,
        testResults: suite.testResults.map(t => ({
          fullName: t.fullName,
          status: t.status,
          duration: t.duration,
          failureMessages: t.failureMessages,
        })),
        perfStats: suite.perfStats,
      })),
    };

    const outputPath = path.isAbsolute(this._outputFile)
      ? this._outputFile
      : path.join(process.cwd(), this._outputFile);

    const dir = path.dirname(outputPath);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });

    fs.writeFileSync(outputPath, JSON.stringify(output, null, 2));
  }
}

module.exports = RegressionReporter;
