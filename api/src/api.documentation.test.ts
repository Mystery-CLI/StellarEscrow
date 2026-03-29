import fs from 'node:fs';
import path from 'node:path';
import { API_ENDPOINT_CONTRACTS } from './contracts';

describe('API documentation', () => {
  const readmePath = path.resolve(__dirname, '..', 'README.md');
  const readme = fs.readFileSync(readmePath, 'utf8');

  it('documents every endpoint contract in the README matrix', () => {
    for (const contract of API_ENDPOINT_CONTRACTS) {
      expect(readme).toContain(contract.path);
      expect(readme).toContain(contract.clientMethod);
    }
  });

  it('documents the dedicated API test commands', () => {
    for (const command of [
      'npm run test:unit',
      'npm run test:endpoints',
      'npm run test:integration',
      'npm run test:contract',
      'npm run test:load',
      'npm run test:docs',
    ]) {
      expect(readme).toContain(command);
    }
  });
});
