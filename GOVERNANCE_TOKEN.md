# Governance Token System

## Overview

The StellarEscrow platform implements a comprehensive governance token system that enables decentralized platform decision-making and fee distribution to token holders. This document details the design, implementation, and usage of the governance token.

## Token Economics

### Supply & Distribution

- **Total Supply**: 1 billion tokens (1,000,000,000 with 7 decimals)
- **Initial Distribution**: Minted to a designated initial holder during initialization
- **Decimals**: 7 (standard for Stellar tokens)
- **Token Type**: Soroban SAC (Stellar Asset Contract)

### Token Allocation Strategy

The governance token should be distributed according to the platform's tokenomics:

1. **Community Allocation** (40%): Distributed to early users and community members
2. **Team & Advisors** (20%): Vested over 4 years
3. **Treasury** (30%): Controlled by governance for platform development
4. **Liquidity Incentives** (10%): For DEX pools and trading incentives

## Voting Mechanics

### Voting Power

- **Base Power**: 1 token = 1 vote
- **Delegation**: Users can delegate voting power to other addresses
- **Delegator Restrictions**: Delegators cannot vote directly; their power is transferred to delegatee
- **Reclamation**: Delegators can undelegate at any time to reclaim voting power

### Proposal Creation

**Requirements**:
- Proposer must hold ≥ 10,000 governance tokens
- Proposer must not have delegated their voting power
- Proposal must specify a valid action

**Proposal Actions**:
1. `UpdateFeeBps(u32)` - Change platform trading fee (0-10000 bps)
2. `UpdateTierConfig(TierConfig)` - Modify tier-based fee discounts
3. `DistributeFees(Address)` - Withdraw accumulated fees to recipient

**Proposal Lifecycle**:
```
Created → Active → Voting Period → Ended → Execution
```

### Voting Process

**Voting Period**: 1,209,600 ledgers (~7 days at 5s/ledger)

**Voting Requirements**:
- Voter must hold governance tokens
- Voter must not have delegated their power
- Voter can only vote once per proposal
- Voting ends after the voting period expires

**Vote Casting**:
```rust
cast_vote(voter, proposal_id, support: bool)
```

**Vote Weight**: Voter's token balance at voting time

### Proposal Execution

**Execution Requirements**:
1. Voting period must have ended
2. Quorum must be met: ≥ 4% of total supply (40 million tokens)
3. Majority must support: votes_for > votes_against
4. Proposal status must be Active

**Execution Process**:
```
1. Check voting period ended
2. Verify quorum (total_votes ≥ 40M)
3. Verify majority (votes_for > votes_against)
4. Execute proposal action
5. Update proposal status to Executed
6. Emit EvProposalExecuted event
```

## Delegation Features

### Delegating Voting Power

```rust
delegate(delegator, delegatee)
```

**Effects**:
- Delegator's voting power transfers to delegatee
- Delegator cannot vote directly
- Delegatee can vote with combined power
- Multiple delegators can delegate to same delegatee

**Use Cases**:
- Users delegate to trusted community members
- Delegation to protocol governance committees
- Delegation to institutional stakeholders

### Undelegating

```rust
undelegate(delegator)
```

**Effects**:
- Delegator reclaims voting power
- Delegator can vote directly again
- Effective immediately

## Fee Distribution

### Accumulated Fees

The platform collects trading fees in USDC:
- Fees accumulate in contract storage
- Per-currency fee tracking for multi-token support
- Fees can be distributed via governance proposal

### Distribution Mechanism

**Via Governance Proposal**:
```rust
ProposalAction::DistributeFees(recipient)
```

**Process**:
1. Governance proposal created to distribute fees
2. Community votes on recipient
3. If passed, fees transferred to recipient
4. Recipient can be:
   - Treasury address for reinvestment
   - Staking contract for token holder rewards
   - Development fund
   - Community initiatives

### Fee Distribution to Token Holders

**Recommended Implementation**:
1. Deploy staking contract accepting governance tokens
2. Create proposal to distribute fees to staking contract
3. Staking contract distributes fees proportionally to stakers
4. Token holders earn yield on governance tokens

## Contract Integration

### Initialization

```rust
// Step 1: Deploy governance token (SAC)
let gov_token = deploy_sac_token();

// Step 2: Initialize governance in escrow contract
initialize_gov_token(
    admin,
    gov_token,
    initial_holder  // Address to receive initial supply
)
```

### Creating a Proposal

```rust
// User with ≥10k tokens creates proposal
create_proposal(
    proposer,
    ProposalAction::UpdateFeeBps(50)  // Change fee to 0.50%
)
// Returns: proposal_id
```

### Voting

```rust
// Token holder votes
cast_vote(
    voter,
    proposal_id,
    support: true  // or false to vote against
)
```

### Executing Proposal

```rust
// After voting period ends
execute_proposal(proposal_id)
// Automatically executes if quorum + majority met
```

### Delegation

```rust
// Delegate voting power
delegate(delegator, delegatee)

// Reclaim voting power
undelegate(delegator)
```

## Query Functions

### Governance State

```rust
// Get governance token address
get_gov_token() -> Address

// Get voting power of address
get_voting_power(voter) -> i128

// Get total voting power
get_total_voting_power() -> i128

// Get quorum requirement
get_quorum_requirement() -> i128  // 40M tokens

// Get proposal threshold
get_proposal_threshold() -> i128  // 10k tokens

// Get voting period
get_voting_period() -> u32  // 1,209,600 ledgers
```

### Proposal Queries

```rust
// Get proposal details
get_proposal_details(proposal_id) -> Proposal

// Get proposal count
get_proposal_count() -> u64

// Check if proposal passed
has_proposal_passed(proposal_id) -> bool

// Get voting summary
get_voting_summary(proposal_id) -> (votes_for, votes_against, voting_ended)
```

## Events

### Governance Events

All governance events are emitted with category `"gov"` for indexer filtering.

#### EvGovTokenInitialized
```rust
{
    token: Address,           // Governance token address
    initial_holder: Address,  // Initial token recipient
    supply: i128,            // Total supply minted
}
```

#### EvProposalCreated
```rust
{
    proposal_id: u64,
    proposer: Address,
}
```

#### EvVoteCast
```rust
{
    proposal_id: u64,
    voter: Address,
    support: bool,           // true = for, false = against
    weight: i128,           // Voter's token balance
}
```

#### EvProposalExecuted
```rust
{
    proposal_id: u64,
}
```

#### EvDelegated
```rust
{
    delegator: Address,
    delegatee: Address,
}
```

#### EvFeesDistributed
```rust
{
    to: Address,            // Recipient
    amount: u64,           // USDC amount
}
```

## Security Considerations

### Vote Manipulation Prevention

1. **One Vote Per Proposal**: `has_voted` tracking prevents double voting
2. **Delegation Exclusivity**: Delegators cannot vote directly
3. **Balance Snapshot**: Voting power based on balance at vote time
4. **Quorum Requirement**: Prevents low-participation decisions

### Proposal Execution Safety

1. **Timelock**: Voting period provides time for community review
2. **Majority Requirement**: Prevents minority control
3. **Quorum Requirement**: Ensures broad participation
4. **Action Validation**: Fee limits (0-10000 bps) prevent invalid changes

### Access Control

1. **Admin Only**: Token initialization requires admin auth
2. **Voter Auth**: Vote casting requires voter signature
3. **Delegator Auth**: Delegation requires delegator signature
4. **Proposal Auth**: Execution requires no special auth (permissionless)

## Usage Examples

### Example 1: Reduce Trading Fee

```rust
// 1. Create proposal to reduce fee from 100 bps to 50 bps
proposal_id = create_proposal(
    proposer,
    ProposalAction::UpdateFeeBps(50)
)

// 2. Community votes
cast_vote(voter1, proposal_id, true)   // Support
cast_vote(voter2, proposal_id, true)   // Support
cast_vote(voter3, proposal_id, false)  // Against

// 3. After voting period, execute
execute_proposal(proposal_id)
// Fee updated to 50 bps
```

### Example 2: Distribute Fees to Treasury

```rust
// 1. Create proposal to distribute accumulated fees
proposal_id = create_proposal(
    proposer,
    ProposalAction::DistributeFees(treasury_address)
)

// 2. Community votes
cast_vote(voter1, proposal_id, true)
cast_vote(voter2, proposal_id, true)

// 3. Execute after voting period
execute_proposal(proposal_id)
// Fees transferred to treasury
```

### Example 3: Delegate Voting Power

```rust
// User delegates to trusted community member
delegate(user_address, trusted_member)

// Trusted member can now vote with user's tokens
cast_vote(trusted_member, proposal_id, true)

// User can reclaim power anytime
undelegate(user_address)
```

## Governance Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Total Supply | 1B tokens | Fixed at initialization |
| Decimals | 7 | Stellar standard |
| Proposal Threshold | 10k tokens | Minimum to create proposal |
| Quorum | 4% (40M tokens) | Minimum participation |
| Voting Period | 7 days | Time to vote |
| Max Fee | 10000 bps | 100% maximum fee |
| Min Fee | 0 bps | 0% minimum fee |

## Future Enhancements

### Planned Features

1. **Timelock Execution**: Delay proposal execution for security review
2. **Vote Escrow (veGOV)**: Longer lock-ups grant more voting power
3. **Quadratic Voting**: Reduce whale influence with quadratic voting
4. **Multi-Sig Governance**: Require multiple signers for critical proposals
5. **Snapshot Voting**: Off-chain voting with on-chain execution
6. **Governance Committees**: Specialized committees for different proposal types

### Governance Evolution

The governance system is designed to evolve:
- New proposal types can be added via contract upgrades
- Voting parameters can be adjusted via proposals
- Delegation mechanisms can be enhanced
- Integration with external governance tools

## Compliance & Auditing

### Event Tracking

All governance actions emit events for:
- Indexer tracking and analytics
- Audit trail and compliance
- Real-time monitoring
- Historical analysis

### Proposal Transparency

- All proposals stored on-chain
- Complete voting history available
- Execution results recorded
- Fee distributions tracked

## Support & Resources

### Documentation
- [Governance Token Spec](./GOVERNANCE_TOKEN.md)
- [Contract API](./api/README.md)
- [Event Schema](./docs/events.md)

### Development
- Governance module: `contract/src/governance.rs`
- Storage layer: `contract/src/storage.rs`
- Event definitions: `contract/src/events.rs`
- Type definitions: `contract/src/types.rs`

### Testing
- Unit tests: `contract/src/test.rs`
- Integration tests: `contract/tests/`
- Governance test suite: `testing/governance/`
