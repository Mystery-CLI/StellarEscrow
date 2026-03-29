/**
 * Config validation CLI
 * Usage: npx ts-node validate.ts [--env development|staging|production]
 *
 * Merges base.toml + environments/<env>.toml, applies env var overrides,
 * then validates the result against schema.json.
 */

import * as fs from 'fs';
import * as path from 'path';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';

// ---------------------------------------------------------------------------
// Minimal TOML parser (no external dep required for simple flat/nested TOML)
// We use the `toml` npm package if available, otherwise fall back to a regex
// approach for the subset we need.
// ---------------------------------------------------------------------------
function parseToml(content: string): Record<string, any> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const toml = require('@iarna/toml');
    return toml.parse(content);
  } catch {
    throw new Error(
      'Missing dependency: run `npm install @iarna/toml ajv ajv-formats` in the config/ directory'
    );
  }
}

function deepMerge(base: Record<string, any>, override: Record<string, any>): Record<string, any> {
  const result = { ...base };
  for (const [key, value] of Object.entries(override)) {
    if (value && typeof value === 'object' && !Array.isArray(value) && typeof result[key] === 'object') {
      result[key] = deepMerge(result[key], value);
    } else {
      result[key] = value;
    }
  }
  return result;
}

/**
 * Apply environment variable overrides.
 * Pattern: STELLAR_ESCROW__<SECTION>__<KEY>=value
 * Example: STELLAR_ESCROW__DATABASE__URL=postgres://...
 */
function applyEnvOverrides(config: Record<string, any>): Record<string, any> {
  const result = JSON.parse(JSON.stringify(config));
  const prefix = 'STELLAR_ESCROW__';

  for (const [envKey, envVal] of Object.entries(process.env)) {
    if (!envKey.startsWith(prefix) || !envVal) continue;

    const parts = envKey.slice(prefix.length).toLowerCase().split('__');
    if (parts.length !== 2) continue;

    const [section, key] = parts;
    if (!result[section]) result[section] = {};

    // Coerce numeric and boolean strings
    if (envVal === 'true') result[section][key] = true;
    else if (envVal === 'false') result[section][key] = false;
    else if (/^\d+$/.test(envVal)) result[section][key] = parseInt(envVal, 10);
    else if (/^\d+\.\d+$/.test(envVal)) result[section][key] = parseFloat(envVal);
    else result[section][key] = envVal;
  }

  return result;
}

function loadConfig(env: string): Record<string, any> {
  const dir = path.resolve(__dirname);
  const base = parseToml(fs.readFileSync(path.join(dir, 'base.toml'), 'utf8'));
  const envFile = path.join(dir, 'environments', `${env}.toml`);

  if (!fs.existsSync(envFile)) {
    throw new Error(`Environment config not found: ${envFile}`);
  }

  const envConfig = parseToml(fs.readFileSync(envFile, 'utf8'));
  const merged = deepMerge(base, envConfig);
  return applyEnvOverrides(merged);
}

function validate(env: string): void {
  console.log(`\nValidating config for environment: ${env}\n`);

  const config = loadConfig(env);
  const schema = JSON.parse(
    fs.readFileSync(path.resolve(__dirname, 'schema.json'), 'utf8')
  );

  const ajv = new Ajv({ allErrors: true });
  addFormats(ajv);
  const validate = ajv.compile(schema);
  const valid = validate(config);

  if (!valid) {
    console.error('Config validation FAILED:\n');
    for (const err of validate.errors ?? []) {
      console.error(`  ✗ ${err.instancePath || '(root)'}: ${err.message}`);
    }
    process.exit(1);
  }

  // Extra semantic checks
  const errors: string[] = [];

  if (env === 'production') {
    if (!config.database?.url) {
      errors.push('database.url must be set for production (use STELLAR_ESCROW__DATABASE__URL)');
    }
    if (!config.stellar?.contract_id) {
      errors.push('stellar.contract_id must be set for production');
    }
    if (config.stellar?.network !== 'mainnet') {
      errors.push('stellar.network must be "mainnet" for production');
    }
  }

  if (errors.length > 0) {
    console.error('Semantic validation FAILED:\n');
    errors.forEach((e) => console.error(`  ✗ ${e}`));
    process.exit(1);
  }

  console.log('✓ Config is valid\n');
  console.log('Resolved config:');
  console.log(JSON.stringify(config, null, 2));
}

// ---------------------------------------------------------------------------
// CLI entry point
// ---------------------------------------------------------------------------
const args = process.argv.slice(2);
const envIndex = args.indexOf('--env');
const env = envIndex !== -1 ? args[envIndex + 1] : (process.env.APP_ENV ?? 'development');

if (!['development', 'staging', 'production'].includes(env)) {
  console.error(`Unknown environment: ${env}. Must be development, staging, or production.`);
  process.exit(1);
}

validate(env);
