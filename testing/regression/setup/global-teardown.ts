import * as fs from 'fs';
import * as path from 'path';

export default async function globalTeardown() {
  const reportsDir = path.join(__dirname, '..', 'reports');
  const metaPath = path.join(reportsDir, 'run-meta.json');

  if (fs.existsSync(metaPath)) {
    const meta = JSON.parse(fs.readFileSync(metaPath, 'utf-8'));
    meta.completedAt = new Date().toISOString();
    meta.durationMs = Date.now() - new Date(meta.startedAt).getTime();
    fs.writeFileSync(metaPath, JSON.stringify(meta, null, 2));
  }

  console.log('[Regression] Suite complete. Reports written to testing/regression/reports/');
}
