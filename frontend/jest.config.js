module.exports = {
  testEnvironment: 'jsdom',
  roots: ['<rootDir>/tests'],
  testMatch: ['**/?(*.)+(spec|test).js'],
  setupFiles: ['jest-fetch-mock/setupJest'],
  setupFilesAfterEnv: [],
};
