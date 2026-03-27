/**
 * seed-test-data.ts
 *
 * Populates the test database with a standard fixture set:
 *   - 5 Users
 *   - 10 Escrows (2 per status: created, funded, completed, disputed, cancelled)
 *   - 3 Assets
 *
 * Schema version guard: aborts if the DB schema_version doesn't match
 * EXPECTED_SCHEMA_VERSION, preventing stale fixtures from corrupting tests.
 *
 * Usage:
 *   DATABASE_URL=postgres://... ts-node tests/scripts/seed-test-data.ts
 */

import { Pool } from 'pg';
import { userList, escrowList, assetFactory, USDC_ASSET, resetEscrowSeq } from '../factories';

const EXPECTED_SCHEMA_VERSION = 1;

async function getSchemaVersion(pool: Pool): Promise<number> {
  try {
    const { rows } = await pool.query<{ version: number }>(
      `SELECT MAX(version)::int AS version FROM schema_migrations`
    );
    return rows[0]?.version ?? 0;
  } catch {
    // schema_migrations table absent — treat as version 0
    return 0;
  }
}

async function ensureTestTables(pool: Pool): Promise<void> {
  await pool.query(`
    CREATE TABLE IF NOT EXISTS test_users (
      address      VARCHAR(56)  PRIMARY KEY,
      display_name VARCHAR(100) NOT NULL,
      kyc_verified BOOLEAN      NOT NULL DEFAULT FALSE,
      created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
    );

    CREATE TABLE IF NOT EXISTS test_escrows (
      trade_id   BIGINT       PRIMARY KEY,
      seller     VARCHAR(56)  NOT NULL,
      buyer      VARCHAR(56)  NOT NULL,
      amount     BIGINT       NOT NULL,
      fee        BIGINT       NOT NULL,
      status     VARCHAR(20)  NOT NULL,
      arbitrator VARCHAR(56),
      created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
    );

    CREATE TABLE IF NOT EXISTS test_assets (
      code       VARCHAR(12)  NOT NULL,
      issuer     VARCHAR(56)  NOT NULL,
      decimals   INT          NOT NULL DEFAULT 7,
      is_native  BOOLEAN      NOT NULL DEFAULT FALSE,
      PRIMARY KEY (code, issuer)
    );
  `);
}

async function seed(): Promise<void> {
  const dbUrl = process.env.DATABASE_URL;
  if (!dbUrl) throw new Error('DATABASE_URL environment variable is required');

  const pool = new Pool({ connectionString: dbUrl });

  try {
    // ── Schema version guard ──────────────────────────────────────────────
    const schemaVersion = await getSchemaVersion(pool);
    if (schemaVersion < EXPECTED_SCHEMA_VERSION) {
      throw new Error(
        `Schema version mismatch: expected >= ${EXPECTED_SCHEMA_VERSION}, got ${schemaVersion}. ` +
        `Run migrations before seeding.`
      );
    }

    await ensureTestTables(pool);

    // ── Users (5) ─────────────────────────────────────────────────────────
    const users = userList(5);
    for (const u of users) {
      await pool.query(
        `INSERT INTO test_users (address, display_name, kyc_verified, created_at)
         VALUES ($1, $2, $3, $4) ON CONFLICT (address) DO NOTHING`,
        [u.address, u.display_name, u.kyc_verified, u.created_at]
      );
    }

    // ── Escrows (10 — 2 per status) ───────────────────────────────────────
    resetEscrowSeq(1);
    const escrows = escrowList(10);
    for (const e of escrows) {
      await pool.query(
        `INSERT INTO test_escrows
           (trade_id, seller, buyer, amount, fee, status, arbitrator, created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (trade_id) DO NOTHING`,
        [e.trade_id, e.seller, e.buyer, e.amount, e.fee,
         e.status, e.arbitrator, e.created_at, e.updated_at]
      );
    }

    // ── Assets (USDC + 2 random) ──────────────────────────────────────────
    const assets = [USDC_ASSET, assetFactory({ code: 'USDT' }), assetFactory({ code: 'XLM', is_native: true, issuer: 'native' })];
    for (const a of assets) {
      await pool.query(
        `INSERT INTO test_assets (code, issuer, decimals, is_native)
         VALUES ($1,$2,$3,$4) ON CONFLICT (code, issuer) DO NOTHING`,
        [a.code, a.issuer, a.decimals, a.is_native]
      );
    }

    console.log(`✅ Seeded: ${users.length} users, ${escrows.length} escrows, ${assets.length} assets`);
  } finally {
    await pool.end();
  }
}

seed().catch((err) => {
  console.error('Seed failed:', err.message);
  process.exit(1);
});
