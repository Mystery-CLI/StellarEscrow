import * as fs from 'fs';
import * as path from 'path';

export default async function globalSetup() {
  // Ensure reports directory exists
  const reportsDir = path.join(__dirname, '..', 'reports');
  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true });
  }

  // Record baseline snapshot timestamp
  const baseline = {
    startedAt: new Date().toISOString(),
    environment: process.env.TEST_ENV || 'development',
    apiBaseUrl: process.env.API_BASE_URL || 'http://localhost:3000',
    commit: process.env.GITHUB_SHA || 'local',
    branch: process.env.GITHUB_REF_NAME || 'local',
  };

  fs.writeFileSync(
    path.join(reportsDir, 'run-meta.json'),
    JSON.stringify(baseline, null, 2)
  );

  console.log(`[Regression] Starting suite against ${baseline.apiBaseUrl}`);
  console.log(`[Regression] Commit: ${baseline.commit} | Branch: ${baseline.branch}`);
}
