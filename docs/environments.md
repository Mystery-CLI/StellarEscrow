# Environment Management (Issue #92)

## Environments

| Environment | Branch | Stellar Network | Purpose |
|---|---|---|---|
| development | any feature branch | TESTNET | Local development |
| staging | `develop` | TESTNET | Pre-production validation |
| production | `main` | PUBLIC (Mainnet) | Live system |

## Configuration Files

| File | Committed | Purpose |
|---|---|---|
| `.env.example` | ✅ yes | Template — copy to `.env` and fill in values |
| `.env.development` | ✅ yes | Non-secret dev defaults |
| `.env.staging` | ✅ yes | Staging placeholders (secrets via CI) |
| `.env.production` | ✅ yes | Production placeholders (secrets via CI) |
| `.env` | ❌ no | Local overrides — gitignored |
| `.env.*.local` | ❌ no | Personal overrides — gitignored |

### Required Variables

| Variable | Description |
|---|---|
| `STELLAR_NETWORK` | `TESTNET` or `PUBLIC` |
| `HORIZON_URL` | Horizon API endpoint |
| `CONTRACT_ID` | Deployed contract address for this environment |
| `DATABASE_URL` | PostgreSQL connection string |
| `NODE_ENV` | `development` / `staging` / `production` |
| `PORT` | HTTP server port (default `3000`) |
| `API_KEYS` | Comma-separated API keys |
| `ADMIN_KEYS` | Comma-separated admin keys |
| `REDIS_URL` | Redis connection string (optional) |
| `LOG_LEVEL` | `debug` / `info` / `warn` / `error` |

## Branching Strategy

```
feature/* ──► develop ──► main
                │              │
             staging        production
           (TESTNET)        (PUBLIC)
```

- Merging to `develop` triggers a staging deployment.
- Merging to `main` does **not** auto-deploy to production.
- Production is promoted manually via the `Environment Promotion` workflow.

## Promotion Workflow

1. Validate staging is healthy (`/health` returns `"status": "ok"`).
2. Trigger **Actions → Environment Promotion** with the image tag to promote.
3. GitHub requires approval from the `production` environment protection rule.
4. The workflow deploys the image, waits for rollout, and re-checks `/health`.
5. On failure the workflow automatically rolls back via `kubectl rollout undo`.

## Health Endpoint

`GET /health` returns:

```json
{
  "status": "ok",
  "node_env": "production",
  "uptime_seconds": 3600,
  "database": "connected",
  "timestamp": "2026-03-26T15:00:00Z"
}
```

Returns HTTP `503` with `"status": "degraded"` when the database is unreachable.

## Rotating Secrets

1. Generate new value (API key, DB password, etc.).
2. Update the secret in **GitHub → Settings → Environments → \<env\> → Secrets**.
3. For the indexer, update `STELLAR_ESCROW__AUTH__API_KEYS` (or the relevant env var override) and redeploy.
4. Revoke the old value only after confirming the new deployment is healthy.
5. For `CONTRACT_ID` rotations (new deployment), update the env var and run a full promotion cycle.

> Never commit real secrets. All sensitive values must live in GitHub Secrets or a secrets manager (e.g., AWS Secrets Manager, HashiCorp Vault).
