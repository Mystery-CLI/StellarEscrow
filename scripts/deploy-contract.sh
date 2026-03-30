#!/usr/bin/env bash
# deploy-contract.sh — Build, deploy, and initialize the StellarEscrow Soroban contract.
#
# Usage:
#   ./scripts/deploy-contract.sh [testnet|mainnet]
#
# Required environment variables:
#   DEPLOY_SECRET_KEY   — Stellar secret key of the deployer account (S...)
#   ADMIN_ADDRESS       — Stellar address to set as contract admin (G...)
#   FEE_BPS             — Platform fee in basis points, e.g. 100 = 1%
#   USDC_TOKEN_ADDRESS  — Stellar address of the USDC token contract
#
# Optional:
#   CONTRACT_WASM_PATH  — Override path to the compiled .wasm (default: auto-detected)
#   CONFIG_OUT          — File to write the deployed contract ID to (default: .env.<network>)
#
# Outputs:
#   Writes CONTRACT_ID=<id> to CONFIG_OUT so downstream services can pick it up.
#
# Example (testnet):
#   export DEPLOY_SECRET_KEY=SXXX...
#   export ADMIN_ADDRESS=GXXX...
#   export FEE_BPS=100
#   export USDC_TOKEN_ADDRESS=CXXX...
#   ./scripts/deploy-contract.sh testnet

set -euo pipefail

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
NETWORK="${1:-testnet}"

if [[ "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" ]]; then
  echo "ERROR: network must be 'testnet' or 'mainnet', got '$NETWORK'" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Validate required environment variables
# ---------------------------------------------------------------------------
MISSING=()
[[ -z "${DEPLOY_SECRET_KEY:-}" ]]   && MISSING+=("DEPLOY_SECRET_KEY")
[[ -z "${ADMIN_ADDRESS:-}" ]]        && MISSING+=("ADMIN_ADDRESS")
[[ -z "${FEE_BPS:-}" ]]              && MISSING+=("FEE_BPS")
[[ -z "${USDC_TOKEN_ADDRESS:-}" ]]   && MISSING+=("USDC_TOKEN_ADDRESS")

if [[ ${#MISSING[@]} -gt 0 ]]; then
  echo "ERROR: Missing required environment variables:" >&2
  for v in "${MISSING[@]}"; do echo "  - $v" >&2; done
  exit 1
fi

# ---------------------------------------------------------------------------
# Derived config
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACT_DIR="$REPO_ROOT/contract"
WASM_DIR="$CONTRACT_DIR/target/wasm32-unknown-unknown/release"
WASM_NAME="stellar_escrow.wasm"
OPTIMIZED_WASM="$WASM_DIR/stellar_escrow.optimized.wasm"
CONTRACT_WASM_PATH="${CONTRACT_WASM_PATH:-$OPTIMIZED_WASM}"
CONFIG_OUT="${CONFIG_OUT:-$REPO_ROOT/.env.$NETWORK}"

case "$NETWORK" in
  testnet)
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    RPC_URL="https://soroban-testnet.stellar.org"
    HORIZON_URL="https://horizon-testnet.stellar.org"
    ;;
  mainnet)
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    RPC_URL="https://soroban-mainnet.stellar.org"
    HORIZON_URL="https://horizon.stellar.org"
    ;;
esac

echo "==> Deploying StellarEscrow contract to $NETWORK"
echo "    Admin:        $ADMIN_ADDRESS"
echo "    Fee (bps):    $FEE_BPS"
echo "    USDC token:   $USDC_TOKEN_ADDRESS"
echo "    Config out:   $CONFIG_OUT"
echo ""

# ---------------------------------------------------------------------------
# Step 1 — Build
# ---------------------------------------------------------------------------
echo "[1/4] Building contract WASM..."
(
  cd "$CONTRACT_DIR"
  cargo build --target wasm32-unknown-unknown --release --quiet
)

echo "[1/4] Optimizing WASM..."
soroban contract optimize \
  --wasm "$WASM_DIR/$WASM_NAME" \
  --wasm-out "$OPTIMIZED_WASM"

echo "      WASM: $CONTRACT_WASM_PATH"

# ---------------------------------------------------------------------------
# Step 2 — Fund deployer on testnet (Friendbot)
# ---------------------------------------------------------------------------
if [[ "$NETWORK" == "testnet" ]]; then
  echo "[2/4] Funding deployer via Friendbot..."
  DEPLOYER_ADDRESS=$(soroban config identity address deployer 2>/dev/null || true)

  # Import key if not already present
  if [[ -z "$DEPLOYER_ADDRESS" ]]; then
    soroban config identity add deployer --secret-key "$DEPLOY_SECRET_KEY"
    DEPLOYER_ADDRESS=$(soroban config identity address deployer)
  fi

  curl -sf "https://friendbot.stellar.org?addr=$DEPLOYER_ADDRESS" -o /dev/null \
    && echo "      Funded $DEPLOYER_ADDRESS" \
    || echo "      Friendbot skipped (account may already exist)"
else
  echo "[2/4] Mainnet — skipping Friendbot (ensure deployer account is funded)"
  soroban config identity add deployer --secret-key "$DEPLOY_SECRET_KEY" 2>/dev/null || true
fi

# ---------------------------------------------------------------------------
# Step 3 — Deploy
# ---------------------------------------------------------------------------
echo "[3/4] Deploying contract..."
CONTRACT_ID=$(soroban contract deploy \
  --wasm "$CONTRACT_WASM_PATH" \
  --source deployer \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE")

if [[ -z "$CONTRACT_ID" ]]; then
  echo "ERROR: Deploy returned empty contract ID" >&2
  exit 1
fi

echo "      Contract ID: $CONTRACT_ID"

# ---------------------------------------------------------------------------
# Step 4 — Initialize
# ---------------------------------------------------------------------------
echo "[4/4] Initializing contract (admin=$ADMIN_ADDRESS, fee_bps=$FEE_BPS)..."
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --usdc_token "$USDC_TOKEN_ADDRESS" \
  --fee_bps "$FEE_BPS"

echo "      Contract initialized."

# ---------------------------------------------------------------------------
# Write contract ID to config file
# ---------------------------------------------------------------------------
echo ""
echo "==> Writing CONTRACT_ID to $CONFIG_OUT"

# Update or append CONTRACT_ID in the target env file
if [[ -f "$CONFIG_OUT" ]]; then
  if grep -q "^CONTRACT_ID=" "$CONFIG_OUT"; then
    # Replace existing line (portable sed)
    sed -i.bak "s|^CONTRACT_ID=.*|CONTRACT_ID=$CONTRACT_ID|" "$CONFIG_OUT"
    rm -f "$CONFIG_OUT.bak"
  else
    echo "CONTRACT_ID=$CONTRACT_ID" >> "$CONFIG_OUT"
  fi
else
  cat > "$CONFIG_OUT" <<EOF
# Auto-generated by deploy-contract.sh — $(date -u +"%Y-%m-%dT%H:%M:%SZ")
STELLAR_NETWORK=$(echo "$NETWORK" | tr '[:lower:]' '[:upper:]')
HORIZON_URL=$HORIZON_URL
CONTRACT_ID=$CONTRACT_ID
EOF
fi

echo ""
echo "✓ Deployment complete"
echo "  Network:     $NETWORK"
echo "  Contract ID: $CONTRACT_ID"
echo "  Config file: $CONFIG_OUT"
