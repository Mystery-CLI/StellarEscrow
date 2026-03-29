module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'jsdom',
  roots: ['<rootDir>/suites'],
  testMatch: ['**/*.a11y.ts', '**/*.a11y.tsx'],
  setupFilesAfterEnv: ['<rootDir>/setup/jest-setup.ts'],
  testTimeout: 30000,
  reporters: [
    'default',
    ['<rootDir>/reporters/a11y-reporter.js', { outputFile: 'reports/a11y-results.json' }],
  ],
};
