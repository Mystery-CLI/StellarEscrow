
# StellarEscrow

Production-ready Soroban smart contract for peer-to-peer marketplace escrow on Stellar blockchain.

## Overview

StellarEscrow is a decentralized escrow system that enables secure peer-to-peer trades using USDC stablecoin. The platform connects buyers and sellers with optional arbitrators who can resolve disputes, with the smart contract managing escrow, fee collection, and settlement.

## Features

- **Escrow-Based Trading**: Secure USDC deposits held in contract until trade completion
- **Arbitrator Network**: Registered arbitrators handle dispute resolution
- **Automated Fee Collection**: Platform fees calculated and accumulated automatically
- **Multi-Status Tracking**: Trades tracked through Created, Funded, Completed, Disputed, and Cancelled states
- **Authorization Security**: Role-based access control for all operations
- **Event Emission**: Comprehensive event logging for off-chain monitoring
- **Dispute Resolution**: Arbitrators can release funds to buyer or seller
- **Cancellation Support**: Sellers can cancel unfunded trades
- **Admin Controls**: Platform fee management and fee withdrawal capabilities

## Architecture

### Core Components

- **lib.rs**: Main contract implementation with all public functions
- **types.rs**: Data structures (Trade, TradeStatus, DisputeResolution)
- **storage.rs**: Persistent and instance storage management
- **errors.rs**: Custom error types for contract operations
- **events.rs**: Event emission functions for monitoring
- **test.rs**: Comprehensive test suite with 20+ test cases

### Storage Model

**Instance Storage**: Admin, USDC token, fee configuration, counters, accumulated fees

**Persistent Storage**: Individual trades, arbitrator registrations

### Fee Calculation

Fees are calculated in basis points (bps):
- 100 bps = 1.0%
- 250 bps = 2.5%
- 500 bps = 5.0%

Formula: `fee = amount * fee_bps / 10000`

## Contract Functions

### Administrative Functions

- `initialize(admin, usdc_token, fee_bps)` - One-time contract initialization
- `register_arbitrator(arbitrator)` - Add arbitrator to approved list (admin only)
- `remove_arbitrator_fn(arbitrator)` - Remove arbitrator from approved list (admin only)
- `update_fee(fee_bps)` - Update platform fee percentage (admin only)
- `withdraw_fees(to)` - Withdraw accumulated fees (admin only)

### User Functions

- `create_trade(seller, buyer, amount, arbitrator)` - Create new trade (seller auth required)
- `fund_trade(trade_id)` - Buyer funds the escrow (buyer auth required)
- `complete_trade(trade_id)` - Seller confirms delivery (seller auth required)
- `confirm_receipt(trade_id)` - Buyer confirms receipt and releases funds (buyer auth required)
- `raise_dispute(trade_id)` - Either party raises dispute (buyer/seller auth required)
- `resolve_dispute(trade_id, resolution)` - Arbitrator resolves dispute (arbitrator auth required)
- `cancel_trade(trade_id)` - Cancel unfunded trade (seller auth required)

### Query Functions

- `get_trade(trade_id)` - Retrieve trade details
- `get_accumulated_fees()` - Check total platform fees collected
- `is_arbitrator_registered(arbitrator)` - Verify arbitrator registration status
- `get_platform_fee_bps()` - Get current fee percentage

## Security Features

- **Authorization Checks**: All state-changing operations require proper authorization
- **Status Validation**: Prevents invalid state transitions
- **Overflow Protection**: Safe math operations with overflow checks
- **Arbitrator Verification**: Only registered arbitrators can resolve disputes
- **Ownership Validation**: Only trade parties can perform specific actions

## Testing

The contract includes comprehensive tests covering:

✅ Initialization and configuration  
✅ Arbitrator registration and removal  
✅ Fee updates and validation  
✅ Trade creation and funding  
✅ Trade completion flow  
✅ Dispute raising and resolution  
✅ Cancellation logic  
✅ Fee withdrawal by admin  
✅ Authorization enforcement  
✅ Error conditions  
✅ Event emission verification  
✅ Multiple trades handling  
✅ Fee calculation accuracy  

Run tests with:

```bash
cargo test
```

## Quick Start

### 1. Build the Contract

```bash
cd StellarEscrow
cargo build --target wasm32-unknown-unknown --release
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/stellar_escrow.wasm
```

### 2. Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_escrow.optimized.wasm \
  --source deployer \
  --network testnet
```

### 3. Initialize

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin <ADMIN_ADDRESS> \
  --usdc_token <USDC_TOKEN_ADDRESS> \
  --fee_bps 100
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for complete deployment instructions.

## Usage Flow

### Admin Setup
1. Deploy contract
2. Initialize with admin address, USDC token, and fee percentage
3. Register trusted arbitrators

### Create Trade
1. Seller creates trade with buyer address, amount, and optional arbitrator
2. Trade ID returned for tracking

### Fund Trade
1. Buyer approves USDC transfer to contract
2. Buyer calls `fund_trade` with trade ID
3. Contract transfers USDC from buyer to escrow

### Complete Trade (Happy Path)
1. Seller delivers goods/services off-chain
2. Seller calls `complete_trade`
3. Buyer calls `confirm_receipt`
4. Contract transfers USDC minus fee to seller
5. Fee added to accumulated platform fees

### Dispute Resolution
1. Either party calls `raise_dispute`
2. Arbitrator investigates off-chain
3. Arbitrator calls `resolve_dispute` with resolution (ReleaseToBuyer or ReleaseToSeller)
4. Contract transfers funds accordingly minus arbitration fee

### Fee Management
1. Admin monitors accumulated fees
2. Admin calls `withdraw_fees` to collect platform revenue

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 1 | AlreadyInitialized | Contract already initialized |
| 2 | NotInitialized | Contract not initialized |
| 3 | InvalidAmount | Amount must be greater than 0 |
| 4 | InvalidFeeBps | Fee must be between 0-10000 bps |
| 5 | ArbitratorNotRegistered | Arbitrator not in approved list |
| 6 | TradeNotFound | Trade ID does not exist |
| 7 | InvalidStatus | Operation not allowed in current status |
| 8 | Overflow | Arithmetic overflow detected |
| 9 | NoFeesToWithdraw | No accumulated fees available |
| 10 | Unauthorized | Caller not authorized for this operation |

## Events

The contract emits events for monitoring:

- `created` - New trade created
- `funded` - Buyer funded escrow
- `complete` - Seller marked as delivered
- `confirm` - Buyer confirmed receipt and settled
- `dispute` - Dispute initiated
- `resolved` - Arbitrator resolved dispute
- `cancel` - Trade cancelled
- `arb_reg` - Arbitrator registered
- `arb_rem` - Arbitrator removed
- `fee_upd` - Platform fee updated
- `fees_out` - Fees withdrawn by admin

## Dependencies

- `soroban-sdk = "21.7.0"` - Latest Soroban SDK

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture and design decisions
- [DEPLOYMENT.md](DEPLOYMENT.md) - Complete deployment guide
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines

## License

MIT

## Support

For issues and questions:
- **GitHub Issues**: Create an issue
- **Stellar Discord**: https://discord.gg/stellar
- **Soroban Docs**: https://soroban.stellar.org

## Contributing

Contributions welcome! Please ensure:
- All tests pass: `cargo test`
- Code follows Rust best practices
- New features include tests
- Documentation is updated

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Roadmap

- [ ] Multi-currency support
- [ ] Batch trade processing
- [ ] Arbitrator reputation system
- [ ] Time-locked escrow options
- [ ] Partial release mechanism
- [ ] Multi-signature arbitration
- [ ] Integration with decentralized identity

---

**StellarEscrow** is a Soroban smart contract built in Rust that enables secure, escrow-based peer-to-peer trading on the Stellar network.

This project demonstrates production-ready patterns for escrow systems, dispute resolution, and fee management on Stellar blockchain.
