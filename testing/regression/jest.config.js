module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/suites'],
  testMatch: ['**/*.regression.ts'],
  moduleFileExtensions: ['ts', 'js', 'json'],
  globalSetup: '<rootDir>/setup/global-setup.ts',
  globalTeardown: '<rootDir>/setup/global-teardown.ts',
  setupFilesAfterFramework: [],
  testTimeout: 30000,
  reporters: [
    'default',
    ['<rootDir>/reporters/regression-reporter.js', {
      outputFile: 'reports/regression-results.json',
    }],
  ],
  collectCoverageFrom: [],
};
