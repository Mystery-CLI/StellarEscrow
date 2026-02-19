# StellarEscrow Architecture

Detailed technical architecture and design decisions for the StellarEscrow smart contract.

## System Overview

StellarEscrow is a decentralized peer-to-peer marketplace escrow system built on Stellar's Soroban smart contract platform. It enables trustless trading between parties with optional third-party arbitration.

## Core Design Principles

1. **Security First**: All operations require proper authorization
2. **State Machine**: Strict state transitions prevent invalid operations
3. **Fee Transparency**: Clear fee calculation and accumulation
4. **Dispute Resolution**: Built-in arbitration mechanism
5. **Event-Driven**: Comprehensive event emission for monitoring

## Architecture Layers

```
┌─────────────────────────────────────────┐
│         User Interface Layer            │
│  (Web/Mobile Apps, CLI, Integrations)   │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Contract Interface Layer         │
│    (Public Functions, Query Methods)     │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Business Logic Layer             │
│  (State Validation, Fee Calculation)     │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Storage Layer                    │
│  (Instance Storage, Persistent Storage)  │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Stellar Blockchain               │
└─────────────────────────────────────────┘
```

## State Machine

### Trade Lifecycle

```
    Created
       ↓
    Funded ←──────┐
       ↓          │
   Completed   Disputed
       ↓          │
   [Confirmed]    │
                  ↓
              [Resolved]

   Cancelled (only from Created)
```

### State Transitions

| From | To | Trigger | Authorization |
|------|----|---------|--------------| 
| Created | Funded | fund_trade | Buyer |
| Created | Cancelled | cancel_trade | Seller |
| Funded | Completed | complete_trade | Seller |
| Funded | Disputed | raise_dispute | Buyer/Seller |
| Completed | Disputed | raise_dispute | Buyer/Seller |
| Completed | [Confirmed] | confirm_receipt | Buyer |
| Disputed | [Resolved] | resolve_dispute | Arbitrator |

## Data Models

### Trade Structure

```rust
pub struct Trade {
    pub id: u64,              // Unique identifier
    pub seller: Address,      // Seller's address
    pub buyer: Address,       // Buyer's address
    pub amount: u64,          // Trade amount in USDC
    pub fee: u64,             // Platform fee amount
    pub arbitrator: Option<Address>, // Optional arbitrator
    pub status: TradeStatus,  // Current state
}
```

### Storage Architecture

**Instance Storage** (Contract-level data):
- `ADMIN`: Administrator address
- `USDC_TOKEN`: USDC token contract address
- `FEE_BPS`: Platform fee in basis points
- `TRADE_COUNTER`: Auto-incrementing trade ID
- `ACCUMULATED_FEES`: Total fees collected
- `INITIALIZED`: Initialization flag

**Persistent Storage** (Entity-level data):
- `TRADE_{id}`: Individual trade records
- `ARB_{address}`: Arbitrator registrations

## Fee Model

### Calculation

```
fee = (amount × fee_bps) / 10000
payout = amount - fee
```

### Examples

| Amount | Fee BPS | Fee | Payout |
|--------|---------|-----|--------|
| 10,000 | 100 (1%) | 100 | 9,900 |
| 10,000 | 250 (2.5%) | 250 | 9,750 |
| 10,000 | 500 (5%) | 500 | 9,500 |

### Fee Accumulation

Fees are accumulated in the contract and can be withdrawn by admin:

```
accumulated_fees += trade.fee
```

## Security Model

### Authorization Layers

1. **Admin Operations**
   - Initialize contract
   - Register/remove arbitrators
   - Update fees
   - Withdraw fees

2. **Seller Operations**
   - Create trade
   - Complete trade
   - Cancel unfunded trade
   - Raise dispute

3. **Buyer Operations**
   - Fund trade
   - Confirm receipt
   - Raise dispute

4. **Arbitrator Operations**
   - Resolve disputes

### Security Features

- **Require Auth**: All state-changing operations require caller authentication
- **Status Guards**: Operations only allowed in specific states
- **Overflow Protection**: Safe arithmetic with checked operations
- **Role Validation**: Arbitrators must be pre-registered
- **Ownership Checks**: Only trade parties can perform actions

## Event System

### Event Types

```rust
// Trade lifecycle events
created(trade_id, seller, buyer, amount)
funded(trade_id)
complete(trade_id)
confirm(trade_id, payout, fee)
cancel(trade_id)

// Dispute events
dispute(trade_id, raised_by)
resolved(trade_id, resolution, recipient)

// Admin events
arb_reg(arbitrator)
arb_rem(arbitrator)
fee_upd(fee_bps)
fees_out(amount, to)
```

### Event Usage

Events enable:
- Off-chain monitoring and indexing
- User notifications
- Analytics and reporting
- Audit trails

## Token Integration

### USDC Token Interface

```rust
pub trait TokenClient {
    fn transfer(from: Address, to: Address, amount: i128);
    fn balance(id: Address) -> i128;
}
```

### Token Flows

1. **Fund Trade**: Buyer → Contract
2. **Confirm Receipt**: Contract → Seller (payout)
3. **Resolve Dispute**: Contract → Winner (payout)
4. **Withdraw Fees**: Contract → Admin

## Error Handling

### Error Hierarchy

```
ContractError
├── AlreadyInitialized (1)
├── NotInitialized (2)
├── InvalidAmount (3)
├── InvalidFeeBps (4)
├── ArbitratorNotRegistered (5)
├── TradeNotFound (6)
├── InvalidStatus (7)
├── Overflow (8)
├── NoFeesToWithdraw (9)
└── Unauthorized (10)
```

### Error Propagation

Errors are propagated using Rust's `Result` type:

```rust
pub fn create_trade(...) -> Result<u64, ContractError>
```

## Gas Optimization

### Storage Efficiency

- Use `u64` for amounts (sufficient for USDC with 7 decimals)
- Use `u32` for basis points (max 10000)
- Minimize storage reads/writes
- Use instance storage for frequently accessed data

### Computation Efficiency

- Early returns on validation failures
- Batch operations where possible
- Minimize external contract calls

## Scalability Considerations

### Current Limitations

- Linear trade ID counter (max: 2^64 - 1)
- No pagination for queries
- Single USDC token support

### Future Enhancements

- Multi-token support
- Batch trade operations
- Trade search/filter capabilities
- Reputation system integration

## Testing Strategy

### Test Coverage

1. **Unit Tests**: Individual function behavior
2. **Integration Tests**: Multi-step workflows
3. **Edge Cases**: Boundary conditions
4. **Security Tests**: Authorization and validation
5. **Event Tests**: Proper event emission

### Test Categories

- Initialization tests
- Admin operation tests
- Trade lifecycle tests
- Dispute resolution tests
- Fee management tests
- Error condition tests

## Deployment Architecture

```
Development → Testnet → Mainnet
     ↓           ↓         ↓
  Local      Soroban   Soroban
  Testing    Testnet   Mainnet
```

### Environment Configuration

- **Development**: Local Soroban instance
- **Testnet**: Public Stellar testnet
- **Mainnet**: Production Stellar network

## Monitoring and Observability

### Key Metrics

- Total trades created
- Total volume processed
- Fees accumulated
- Dispute rate
- Average trade completion time

### Event Monitoring

Monitor contract events for:
- Trade creation rate
- Dispute frequency
- Fee withdrawals
- Arbitrator activity

## Upgrade Strategy

Since Soroban contracts are immutable:

1. Deploy new contract version
2. Pause old contract (if possible)
3. Migrate critical state
4. Update frontend/integrations
5. Deprecate old contract

## Integration Patterns

### Frontend Integration

```javascript
// Create trade
const tradeId = await contract.create_trade({
  seller: sellerAddress,
  buyer: buyerAddress,
  amount: 10000,
  arbitrator: arbitratorAddress
});

// Fund trade
await contract.fund_trade({ trade_id: tradeId });
```

### Backend Integration

- Event indexing for trade history
- Notification system for status changes
- Analytics dashboard for metrics
- Admin panel for management

## Comparison with SwiftRemit

| Feature | StellarEscrow | SwiftRemit |
|---------|---------------|------------|
| Use Case | P2P Trading | Remittances |
| Parties | Buyer/Seller | Sender/Agent |
| Dispute | Arbitrator | N/A |
| Cancellation | Unfunded only | Pending only |
| Payout | On confirmation | On agent confirm |
| Fee Model | Same | Same |

## Future Roadmap

1. **Phase 1**: Core escrow functionality ✅
2. **Phase 2**: Multi-token support
3. **Phase 3**: Reputation system
4. **Phase 4**: Partial releases
5. **Phase 5**: Multi-sig arbitration
6. **Phase 6**: Time-locked escrow

## References

- [Soroban Documentation](https://soroban.stellar.org)
- [Stellar SDK](https://github.com/stellar/rs-soroban-sdk)
- [Smart Contract Best Practices](https://soroban.stellar.org/docs/learn/best-practices)
