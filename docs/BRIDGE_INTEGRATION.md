# Cross-Chain Bridge Integration Guide

## Overview

StellarEscrow now supports cross-chain trades through multiple bridge protocols. This enables users to initiate trades on one blockchain and settle them on Stellar, with automatic validation and failure handling.

## Architecture

### Components

1. **Bridge Contract Module** (`contract/src/bridge.rs`)
   - Provider registration and management
   - Cross-chain trade state tracking
   - Attestation validation
   - Failure handling and rollback

2. **Bridge Service** (`indexer/src/bridge_service/`)
   - Attestation processing
   - Provider coordination
   - Validation logic
   - Retry and recovery mechanisms

3. **Bridge API** (`api/src/bridge.ts`)
   - REST endpoints for bridge operations
   - Provider management
   - Trade status queries
   - Health monitoring

### Supported Bridge Providers

- **Wormhole**: EVM chains (Ethereum, Polygon, Avalanche, Arbitrum, Optimism)
- **IBC**: Cosmos ecosystem (Cosmos, Osmosis, Juno)
- **Axelar**: Multi-chain (Ethereum, Polygon, Avalanche, Fantom)
- **LayerZero**: Coming soon
- **Custom**: Extensible for custom implementations

## Trade Flow

### Cross-Chain Trade Lifecycle

```
1. User initiates trade on source chain
   ↓
2. Bridge provider locks/burns tokens
   ↓
3. Bridge oracle submits attestation to Stellar
   ↓
4. Contract validates attestation
   ↓
5. Trade moves to Funded state
   ↓
6. Normal trade flow (completion, dispute resolution)
   ↓
7. On completion, funds released on destination chain
```

### State Transitions

```
Created → AwaitingBridge → Funded → Completed
                    ↓
                BridgeFailed → (Rollback)
```

## API Usage

### Register a Bridge Provider

```typescript
const bridgeApi = new BridgeApi(client);

await bridgeApi.registerProvider({
  name: 'wormhole',
  oracle_address: 'GXXXXXX...',
  is_active: true,
  fee_bps: 50,
  supported_chains: 'ethereum,polygon,avalanche',
  max_trade_amount: '1000000000000', // 1M USDC
  min_trade_amount: '1000000', // 1 USDC
});
```

### Create a Cross-Chain Trade

```typescript
// User initiates trade on Ethereum
const crossChainTrade = await bridgeApi.createCrossChainTrade(
  tradeId,
  'ethereum',
  '0x1234...', // source tx hash
  'wormhole',
  'attestation-123',
  '50000000' // bridge fee in stroops
);
```

### Submit Bridge Attestation

```typescript
const attestation: BridgeAttestation = {
  attestation_id: 'att-123',
  trade_id: 1,
  source_chain: 'ethereum',
  source_tx_hash: '0x1234...',
  amount: '1000000000',
  recipient: 'GXXXXXX...',
  timestamp: Math.floor(Date.now() / 1000),
  signature: '0xabcd...', // Bridge provider signature
  provider: 'wormhole',
};

const validation = await bridgeApi.submitAttestation(attestation);
if (validation.valid) {
  console.log('Attestation confirmed with', validation.confirmations, 'confirmations');
}
```

### Handle Failures

```typescript
// Check if trade should be retried
const trade = await bridgeApi.getCrossChainTrade(tradeId);

if (trade.attestation_status === 'failed' && trade.retry_count < 3) {
  await bridgeApi.retryAttestation(tradeId);
}

// Rollback if unrecoverable
if (trade.retry_count >= 3) {
  await bridgeApi.rollbackTrade(tradeId, 'Max retries exceeded');
}
```

### Monitor Provider Health

```typescript
// Get specific provider status
const status = await bridgeApi.getProviderStatus('wormhole');
console.log(`Wormhole: ${status.healthy ? 'healthy' : 'unhealthy'} (${status.latency_ms}ms)`);

// Get all provider statuses
const allStatuses = await bridgeApi.getAllProviderStatuses();
```

## Smart Contract Integration

### Register Bridge Provider (Admin)

```rust
use stellar_escrow::bridge::{BridgeProvider, BridgeProviderConfig, register_bridge_provider};

let config = BridgeProviderConfig {
    provider: BridgeProvider::Wormhole,
    oracle_address: oracle_addr.clone(),
    is_active: true,
    fee_bps: 50,
    supported_chains: String::from_small_copy(env, "ethereum,polygon"),
    max_trade_amount: 1_000_000_000_000,
    min_trade_amount: 1_000_000,
};

register_bridge_provider(env, config)?;
```

### Create Cross-Chain Trade

```rust
use stellar_escrow::bridge::create_cross_chain_trade;

let cross_chain_trade = create_cross_chain_trade(
    env,
    trade_id,
    String::from_small_copy(env, "ethereum"),
    String::from_small_copy(env, "0x1234..."),
    BridgeProvider::Wormhole,
    String::from_small_copy(env, "att-123"),
    bridge_fee,
)?;
```

### Validate Attestation

```rust
use stellar_escrow::bridge::{validate_bridge_attestation, BridgeAttestation};

let attestation = BridgeAttestation {
    attestation_id: String::from_small_copy(env, "att-123"),
    trade_id: 1,
    source_chain: String::from_small_copy(env, "ethereum"),
    source_tx_hash: String::from_small_copy(env, "0x1234..."),
    amount: 1_000_000_000,
    recipient: buyer.clone(),
    timestamp: env.ledger().timestamp(),
    signature: signature_bytes,
    provider: BridgeProvider::Wormhole,
};

let validation = validate_bridge_attestation(env, &attestation)?;
if validation.valid {
    // Proceed with trade
}
```

## Error Handling

### Bridge-Specific Errors

| Error Code | Error | Meaning |
|-----------|-------|---------|
| 80 | BridgeProviderNotFound | Provider not registered |
| 81 | BridgeProviderAlreadyRegistered | Provider already exists |
| 82 | BridgeProviderLimitExceeded | Max providers reached |
| 83 | BridgeTradeNotFound | Cross-chain trade not found |
| 84 | BridgeTradeExpired | Attestation timeout exceeded |
| 85 | BridgeTradeNotExpired | Trade not yet expired |
| 86 | BridgeRetryLimitExceeded | Max retries exceeded |
| 87 | BridgeAttestationInvalid | Invalid attestation |
| 88 | BridgeAttestationExpired | Attestation too old |
| 89 | BridgeAmountOutOfRange | Amount outside limits |
| 90 | BridgeChainNotSupported | Source chain not supported |
| 91 | BridgeOracleNotAuthorized | Oracle not authorized |
| 92 | BridgePaused | Bridge operations paused |
| 93 | BridgeSignatureInvalid | Invalid signature |
| 94 | BridgeNonceAlreadyUsed | Nonce already used |

### Retry Logic

- **Max Retries**: 3 attempts
- **Retry Delay**: 5 minutes between attempts
- **Timeout**: 1 hour for attestation confirmation
- **Finality**: 12 block confirmations required

### Failure Recovery

1. **Automatic Retry**: Failed attestations retry after 5 minutes
2. **Manual Retry**: Admin can trigger retry via API
3. **Rollback**: After max retries, trade is rolled back
4. **Refund**: User receives refund on source chain

## Database Schema

### cross_chain_trades

```sql
CREATE TABLE cross_chain_trades (
    id UUID PRIMARY KEY,
    trade_id BIGINT UNIQUE,
    source_chain VARCHAR(50),
    dest_chain VARCHAR(50),
    source_tx_hash VARCHAR(100),
    bridge_provider VARCHAR(50),
    attestation_id VARCHAR(100) UNIQUE,
    attestation_status VARCHAR(20),
    attestation_timestamp TIMESTAMPTZ,
    retry_count INTEGER,
    current_confirmations INTEGER,
    bridge_fee BIGINT,
    created_at TIMESTAMPTZ
);
```

### bridge_attestations

```sql
CREATE TABLE bridge_attestations (
    id UUID PRIMARY KEY,
    attestation_id VARCHAR(100) UNIQUE,
    trade_id BIGINT,
    source_chain VARCHAR(50),
    source_tx_hash VARCHAR(100),
    amount BIGINT,
    recipient VARCHAR(100),
    timestamp TIMESTAMPTZ,
    signature BYTEA,
    provider VARCHAR(50),
    verified BOOLEAN,
    created_at TIMESTAMPTZ
);
```

## Events

### Bridge Events

- `EvBridgeProviderRegistered` - New provider registered
- `EvBridgeProviderDeactivated` - Provider deactivated
- `EvBridgeTradeCreated` - Cross-chain trade initiated
- `EvBridgeAttestationReceived` - Attestation submitted
- `EvBridgeAttestationConfirmed` - Attestation confirmed
- `EvBridgeAttestationFailed` - Attestation failed
- `EvBridgeTradeRolledBack` - Trade rolled back
- `EvBridgePaused` - Bridge paused
- `EvBridgeResumed` - Bridge resumed

## Security Considerations

### Attestation Validation

1. **Signature Verification**: All attestations must be cryptographically signed
2. **Timestamp Validation**: Attestations must be recent (< 1 hour old)
3. **Duplicate Prevention**: Each attestation ID processed only once
4. **Amount Limits**: Enforced per provider and globally

### Provider Management

1. **Oracle Authorization**: Only registered oracles can submit attestations
2. **Provider Activation**: Providers can be deactivated for maintenance
3. **Fee Limits**: Bridge fees capped per provider
4. **Chain Whitelisting**: Only approved source chains accepted

### Failure Handling

1. **Automatic Rollback**: Failed trades automatically rolled back
2. **Refund Guarantee**: Users refunded on source chain
3. **Audit Trail**: All bridge operations logged
4. **Emergency Pause**: Bridge can be paused by admin

## Monitoring

### Health Checks

```typescript
// Monitor provider health
const statuses = await bridgeApi.getAllProviderStatuses();
statuses.forEach(status => {
  if (!status.healthy) {
    console.warn(`Provider ${status.provider} is unhealthy`);
  }
});
```

### Metrics

```typescript
// Get bridge statistics
const stats = await bridgeApi.getBridgeStats();
console.log(`Total cross-chain trades: ${stats.total_cross_chain_trades}`);
console.log(`Success rate: ${stats.successful_attestations / stats.total_cross_chain_trades * 100}%`);
console.log(`Average confirmation time: ${stats.average_confirmation_time_secs}s`);
```

## Testing

### Unit Tests

```bash
# Test bridge module
cargo test --package contract bridge::tests

# Test bridge service
cargo test --package indexer bridge_service::tests
```

### Integration Tests

```bash
# Test cross-chain trade flow
npm run test:bridge:integration

# Test provider failover
npm run test:bridge:failover

# Test retry logic
npm run test:bridge:retry
```

## Deployment

### Prerequisites

1. Deploy bridge provider oracles on source chains
2. Register providers in StellarEscrow contract
3. Configure provider endpoints in indexer
4. Set up monitoring and alerting

### Deployment Steps

1. **Deploy Contract**: Update contract with bridge module
2. **Run Migrations**: Execute bridge database migrations
3. **Initialize Providers**: Register bridge providers
4. **Start Indexer**: Launch bridge service
5. **Monitor**: Verify provider health checks

## Troubleshooting

### Attestation Not Confirming

1. Check provider health: `bridgeApi.getProviderStatus(provider)`
2. Verify source chain confirmations
3. Check amount is within limits
4. Verify oracle is authorized

### Provider Unhealthy

1. Check provider RPC endpoint
2. Verify network connectivity
3. Check provider service status
4. Review provider logs

### Trade Stuck in AwaitingBridge

1. Check attestation status
2. Verify retry count < 3
3. Trigger manual retry if needed
4. Rollback if unrecoverable

## Future Enhancements

- [ ] LayerZero provider implementation
- [ ] Liquidity pool integration
- [ ] Atomic swaps across chains
- [ ] Governance-controlled provider fees
- [ ] Advanced retry strategies
- [ ] Multi-hop bridge routing
