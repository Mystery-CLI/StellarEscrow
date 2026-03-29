# Stellar Escrow Event Indexer

A high-performance event indexing service for the Stellar Escrow smart contract that provides efficient querying and real-time event streaming.

## Features

- **Real-time Event Monitoring**: Continuously monitors Stellar network for contract events
- **Efficient Storage**: PostgreSQL-based storage with optimized indexes
- **REST API**: Query events by type, trade ID, ledger range, and more
- **WebSocket Streaming**: Real-time event broadcasting to connected clients
- **Event Replay**: Replay historical events for catch-up or analysis
- **Configurable**: Flexible configuration for different networks and contracts
- **Redis Caching**: Read-through caching, invalidation, monitoring, and warming for hot API paths
- **Background Jobs**: Redis-backed queue with priorities, scheduling, retries, and monitoring

## Architecture

### Components

- **Event Monitor**: Polls Stellar Horizon API for contract effects and parses them into structured events
- **Database Layer**: PostgreSQL storage with optimized queries and indexes
- **REST API**: HTTP endpoints for event querying and replay
- **WebSocket Server**: Real-time event streaming to connected clients
- **Configuration**: TOML-based configuration management

### Event Types

The indexer monitors and stores the following contract events:

- `trade_created` - New trade creation
- `trade_funded` - Buyer funds the escrow
- `trade_completed` - Seller marks trade as completed
- `trade_confirmed` - Buyer confirms receipt and releases funds
- `dispute_raised` - Dispute raised by a party
- `dispute_resolved` - Arbitrator resolves a dispute
- `trade_cancelled` - Trade cancellation
- `arbitrator_registered` - New arbitrator registration
- `arbitrator_removed` - Arbitrator removal
- `fee_updated` - Platform fee update
- `fees_withdrawn` - Fee withdrawal by admin

## Setup

### Prerequisites

- Rust 1.70+
- PostgreSQL 13+
- Access to Stellar Horizon API

### Installation

1. Clone the repository
2. Navigate to the indexer directory: `cd indexer`
3. Copy and modify configuration: `cp config.toml config.local.toml`
4. Update database URL and contract ID in `config.local.toml`

### Database Setup

```bash
# Create database
createdb stellar_escrow

# Run migrations
sqlx migrate run
```

### Running

```bash
# Development
cargo run -- --config config.local.toml

# Production
cargo build --release
./target/release/stellar-escrow-indexer --config config.local.toml
```

## API Documentation

### REST Endpoints

#### Versioned API (v1) and Authentication
- API versioning prefix: `/api/v1/*`
- Auth: `Authorization: Bearer <API_KEY>` or `x-api-key: <API_KEY>`
- Admin routes (req. `admin` key): `/admin/*` and `/api/v1/admin/*`

#### Health Check
```
GET /health
```

#### Get Events
```
GET /events
Query Parameters:
- limit: number (default: 50)
- offset: number (default: 0)
- event_type: string
- trade_id: number
- from_ledger: number
- to_ledger: number
```

#### Get Event by ID
```
GET /events/{id}
```

#### Get Events by Trade ID
```
GET /events/trade/{trade_id}
```

#### Get Events by Type
```
GET /events/type/{event_type}
```

#### Replay Events
```
POST /events/replay
Content-Type: application/json

{
  "from_ledger": 123456,
  "to_ledger": 123500  // optional
}
```

### WebSocket

Connect to `/ws` for real-time event streaming. Events are broadcast as JSON messages:

```json
{
  "event_type": "trade_created",
  "data": {
    "trade_id": 123,
    "seller": "GA...",
    "buyer": "GB...",
    "amount": 10000000
  },
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## Configuration

### Server Configuration
```toml
[server]
port = 3000
```

### Database Configuration
```toml
[database]
url = "postgres://user:password@localhost/stellar_escrow"
```

### Cache Configuration
```toml
[cache]
redis_url = "redis://:password@localhost:6379"
default_ttl_secs = 30
events_ttl_secs = 10
```

### Stellar Configuration
```toml
[stellar]
network = "testnet"  # or "mainnet"
contract_id = "CA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ"
horizon_url = "https://horizon-testnet.stellar.org"
start_ledger = 123456  # optional: start from specific ledger
poll_interval_seconds = 5
```

## Development

### Running Tests
```bash
cargo test
```

### Database Migrations
```bash
# Create new migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Building for Production
```bash
cargo build --release
```

## Monitoring

The service provides health check endpoints and comprehensive logging. Monitor:

- Event processing rate
- Database connection health
- WebSocket connection count
- Memory usage
- Horizon API response times

## Security Considerations

- Use HTTPS in production
- Implement rate limiting
- Secure database credentials
- Monitor for unusual event patterns
- Regular security updates

## License

This project is licensed under the MIT License.
