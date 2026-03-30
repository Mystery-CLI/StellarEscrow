module.exports = {
  testEnvironment: 'jsdom',
  roots: ['<rootDir>/tests'],
  testMatch: ['**/?(*.)+(spec|test).js'],
  setupFiles: ['jest-fetch-mock/setupJest'],
  setupFilesAfterEnv: ['<rootDir>/tests/setup.js'],
  verbose: false,     
  silent: true,        
};