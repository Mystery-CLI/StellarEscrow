# Centralized Configuration Service

Manages configuration for all StellarEscrow services across environments.

## Structure

```
config/
├── base.toml                  # Shared defaults across all environments
├── environments/
│   ├── development.toml       # Local dev overrides
│   ├── staging.toml           # Staging overrides
│   └── production.toml        # Production overrides
├── schema.json                # JSON Schema for validation
├── validate.ts                # Config validation CLI
├── deploy.sh                  # Config deployment script
├── versions.json              # Config version registry
└── README.md
```

## Usage

### Select environment
Set `APP_ENV=development|staging|production` (default: `development`).

### Indexer
Config is merged: `base.toml` → `environments/<env>.toml` → environment variable overrides.

Any field can be overridden via env var using the pattern:
```
STELLAR_ESCROW__<SECTION>__<KEY>=value
```
Examples:
```bash
STELLAR_ESCROW__DATABASE__URL=postgres://...
STELLAR_ESCROW__SERVER__PORT=8080
STELLAR_ESCROW__STELLAR__NETWORK=mainnet
```

### API / Frontend
Set `STELLAR_ESCROW_API_BASE_URL`, `STELLAR_ESCROW_WS_URL`, etc.

### Validation
```bash
cd config
npx ts-node validate.ts --env production
```

### Deployment
```bash
cd config
./deploy.sh --env staging --service indexer
```
