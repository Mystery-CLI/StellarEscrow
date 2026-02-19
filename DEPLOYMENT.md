# StellarEscrow Deployment Guide

Complete guide for deploying and managing the StellarEscrow smart contract on Stellar.

## Prerequisites

- Rust toolchain (1.70+)
- Soroban CLI
- Stellar account with XLM for gas fees
- USDC token contract address

## Installation

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

### 2. Install Soroban CLI

```bash
cargo install --locked soroban-cli
```

### 3. Configure Stellar Network

```bash
# Testnet
soroban network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Mainnet (when ready)
soroban network add \
  --global mainnet \
  --rpc-url https://soroban-mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

## Build Contract

```bash
# Clone repository
git clone <your-repo-url>
cd stellar-escrow

# Build optimized WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize contract size
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.wasm
```

This produces `stellar_escrow.optimized.wasm` ready for deployment.

## Deploy to Testnet

### 1. Create/Import Identity

```bash
# Generate new identity
soroban keys generate deployer --network testnet

# Or import existing secret key
soroban keys add deployer --secret-key
```

### 2. Fund Account

Get testnet XLM from the friendbot:

```bash
soroban keys address deployer | xargs -I {} curl "https://friendbot.stellar.org?addr={}"
```

### 3. Deploy Contract

```bash
CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.optimized.wasm \
  --source deployer \
  --network testnet)

echo "Contract deployed at: $CONTRACT_ID"
```

### 4. Initialize Contract

```bash
# Get your admin address
ADMIN=$(soroban keys address deployer)

# Use testnet USDC address (example)
USDC_TOKEN="CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"

# Initialize with 1% fee (100 basis points)
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin $ADMIN \
  --usdc_token $USDC_TOKEN \
  --fee_bps 100
```

## Contract Management

### Register Arbitrator

```bash
ARBITRATOR="GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  register_arbitrator \
  --arbitrator $ARBITRATOR
```

### Update Platform Fee

```bash
# Update to 2.5% (250 basis points)
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  update_fee \
  --fee_bps 250
```

### Withdraw Accumulated Fees

```bash
FEE_RECIPIENT="GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  withdraw_fees \
  --to $FEE_RECIPIENT
```

## User Operations

### Create Trade

```bash
SELLER="GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
BUYER="GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
ARBITRATOR="GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

TRADE_ID=$(soroban contract invoke \
  --id $CONTRACT_ID \
  --source seller-key \
  --network testnet \
  -- \
  create_trade \
  --seller $SELLER \
  --buyer $BUYER \
  --amount 1000000 \
  --arbitrator $ARBITRATOR)

echo "Trade created with ID: $TRADE_ID"
```

### Fund Trade

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source buyer-key \
  --network testnet \
  -- \
  fund_trade \
  --trade_id $TRADE_ID
```

### Complete Trade

```bash
# Seller marks as delivered
soroban contract invoke \
  --id $CONTRACT_ID \
  --source seller-key \
  --network testnet \
  -- \
  complete_trade \
  --trade_id $TRADE_ID

# Buyer confirms receipt
soroban contract invoke \
  --id $CONTRACT_ID \
  --source buyer-key \
  --network testnet \
  -- \
  confirm_receipt \
  --trade_id $TRADE_ID
```

### Raise and Resolve Dispute

```bash
# Raise dispute
soroban contract invoke \
  --id $CONTRACT_ID \
  --source buyer-key \
  --network testnet \
  -- \
  raise_dispute \
  --trade_id $TRADE_ID

# Resolve dispute (arbitrator only)
soroban contract invoke \
  --id $CONTRACT_ID \
  --source arbitrator-key \
  --network testnet \
  -- \
  resolve_dispute \
  --trade_id $TRADE_ID \
  --resolution ReleaseToBuyer
```

## Query Operations

### Get Trade Details

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  get_trade \
  --trade_id 1
```

### Check Accumulated Fees

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  get_accumulated_fees
```

### Verify Arbitrator

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  is_arbitrator_registered \
  --arbitrator $ARBITRATOR
```

### Get Platform Fee

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- \
  get_platform_fee_bps
```

## Mainnet Deployment

When ready for production:

1. Audit contract code thoroughly
2. Run comprehensive tests: `cargo test`
3. Deploy to testnet and test all flows
4. Get security audit from reputable firm
5. Deploy to mainnet using same steps as testnet but with `--network mainnet`

## Monitoring

### Watch Events

```bash
# Monitor contract events
soroban events --id $CONTRACT_ID --network testnet --start-ledger <LEDGER>
```

### Check Contract Balance

```bash
soroban contract invoke \
  --id $USDC_TOKEN \
  --source deployer \
  --network testnet \
  -- \
  balance \
  --id $CONTRACT_ID
```

## Troubleshooting

### Common Issues

1. **Insufficient Balance**: Ensure accounts have enough XLM for gas
2. **Authorization Failed**: Verify correct signer is being used
3. **Invalid Status**: Check trade status before operations
4. **Contract Not Found**: Verify contract ID is correct

### Debug Mode

Build with debug symbols:

```bash
cargo build --target wasm32-unknown-unknown --release --features testutils
```

## Security Considerations

- Store private keys securely (use hardware wallets for mainnet)
- Implement multi-sig for admin operations
- Monitor contract for unusual activity
- Set reasonable fee limits
- Regularly audit arbitrator list
- Implement rate limiting off-chain

## Upgrade Strategy

Soroban contracts are immutable. For upgrades:

1. Deploy new contract version
2. Migrate state if needed
3. Update frontend to use new contract
4. Deprecate old contract gracefully

## Support

- Stellar Discord: https://discord.gg/stellar
- Soroban Docs: https://soroban.stellar.org
- GitHub Issues: <your-repo-url>/issues

## License

MIT
