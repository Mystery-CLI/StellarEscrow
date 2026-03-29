/**
 * teardown.ts
 *
 * Global Jest teardown (and standalone CLI script) that truncates all test_*
 * tables to prevent data leakage between test suite runs.
 *
 * Jest wires this automatically via jest.globalTeardown in package.json.
 * Run standalone: DATABASE_URL=postgres://... ts-node tests/scripts/teardown.ts
 */

import { Pool } from 'pg';

const TEST_TABLES = ['test_escrows', 'test_users', 'test_assets'] as const;

async function teardown(): Promise<void> {
  const dbUrl = process.env.DATABASE_URL;
  if (!dbUrl) {
    console.warn('teardown: DATABASE_URL not set — skipping DB cleanup');
    return;
  }

  const pool = new Pool({ connectionString: dbUrl });
  try {
    // TRUNCATE in one statement; CASCADE handles FK dependencies
    await pool.query(`TRUNCATE TABLE ${TEST_TABLES.join(', ')} RESTART IDENTITY CASCADE`);
    console.log(`🧹 Truncated: ${TEST_TABLES.join(', ')}`);
  } catch (err: any) {
    // Tables may not exist in CI environments that skip seeding — not fatal
    console.warn('teardown warning:', err.message);
  } finally {
    await pool.end();
  }
}

// Jest globalTeardown export
module.exports = teardown;

// Standalone CLI entry point
if (require.main === module) {
  teardown().catch((err) => {
    console.error('Teardown failed:', err.message);
    process.exit(1);
  });
}
