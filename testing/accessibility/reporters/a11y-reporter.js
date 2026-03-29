'use strict';

const fs = require('fs');
const path = require('path');

class A11yReporter {
  constructor(_globalConfig, options) {
    this._outputFile = (options || {}).outputFile || 'reports/a11y-results.json';
  }

  onRunComplete(_contexts, results) {
    const tests = [];
    for (const suite of results.testResults) {
      for (const t of suite.testResults) {
        const wcagMatch = t.fullName.match(/WCAG\s+([\d.]+)/i);
        tests.push({
          suite: path.basename(suite.testFilePath || ''),
          name: t.fullName,
          wcag_criterion: wcagMatch ? wcagMatch[1] : null,
          status: t.status,
          duration_ms: t.duration || 0,
          error: t.failureMessages?.join('\n') || null,
        });
      }
    }

    const output = {
      meta: {
        timestamp: new Date().toISOString(),
        wcag_level: 'AA',
        commit: process.env.GITHUB_SHA || 'local',
      },
      summary: {
        total: results.numTotalTests,
        passed: results.numPassedTests,
        failed: results.numFailedTests,
        pass_rate: results.numTotalTests > 0
          ? parseFloat(((results.numPassedTests / results.numTotalTests) * 100).toFixed(1))
          : 0,
      },
      tests,
    };

    const outputPath = path.isAbsolute(this._outputFile)
      ? this._outputFile
      : path.join(process.cwd(), this._outputFile);

    const dir = path.dirname(outputPath);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(outputPath, JSON.stringify(output, null, 2));
  }
}

module.exports = A11yReporter;
