module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/scenarios'],
  testMatch: ['**/*.uat.ts'],
  testTimeout: 60000,
  reporters: [
    'default',
    ['<rootDir>/reporters/uat-reporter.js', { outputFile: 'reports/uat-results.json' }],
  ],
};
