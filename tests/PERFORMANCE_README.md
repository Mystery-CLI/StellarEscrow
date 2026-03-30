# StellarEscrow Performance Testing Suite

This directory contains comprehensive performance testing tools for the StellarEscrow platform.

## Overview

The performance testing suite includes:

- **Load Testing**: Sustained traffic simulation using Artillery
- **Stress Testing**: System limits testing with high concurrency
- **Spike Testing**: Sudden traffic surge simulation
- **API Benchmarks**: Jest-based performance baselines
- **Scalability Testing**: Multi-threaded performance analysis
- **Performance Monitoring**: Real-time metrics and alerting
- **Contract Performance**: Smart contract efficiency testing

## Directory Structure

```
tests/
├── load/                    # Artillery load testing configurations
│   ├── load-test.yml       # Sustained load testing
│   ├── stress-test.yml     # High-concurrency stress testing
│   └── spike-test.yml      # Traffic spike simulation
├── performance/            # Jest performance test utilities
└── reports/                # Generated performance reports
```

## Quick Start

### Prerequisites

- Node.js 20+
- Rust (for contract tests)
- Docker (for test services)
- Artillery CLI: `npm install -g artillery`

### Running Tests

```bash
# Run comprehensive performance test suite
npm run test:performance:comprehensive

# Run individual test types
npm run test:performance:load      # Load testing
npm run test:performance:stress    # Stress testing
npm run test:performance:spike     # Spike testing
npm run test:performance           # API benchmarks
```

### Smart Contract Performance

```bash
# Run contract stress tests
./scripts/stress-test.sh

# Run with release build (faster, more accurate)
./scripts/stress-test.sh --release
```

## Test Configurations

### Load Testing (`load-test.yml`)

Simulates normal usage patterns with gradual load increase:
- Warm-up phase: 10 req/sec for 60s
- Sustained load: 10 req/sec for 300s
- Increased load: 20 req/sec for 60s
- Cool-down: 5 req/sec for 60s

### Stress Testing (`stress-test.yml`)

Tests system limits under extreme conditions:
- Burst traffic up to 100 req/sec
- High concurrency scenarios
- Degraded upstream conditions

### Spike Testing (`spike-test.yml`)

Simulates sudden traffic surges:
- Baseline: 5 req/sec
- Spike: 200 req/sec for 30s
- Recovery: 5 req/sec
- Second spike: 300 req/sec for 30s

## Metrics and Monitoring

### Prometheus Metrics

Performance metrics are exported at `/metrics` endpoint:

```bash
curl http://localhost:3000/metrics
```

### Grafana Dashboard

View real-time performance data in Grafana:
- Dashboard: "StellarEscrow Performance Testing"
- Metrics: Latency, throughput, error rates, resource usage

### Key Metrics

- **Latency**: P50, P95, P99 response times
- **Throughput**: Requests per second
- **Error Rate**: Percentage of failed requests
- **CPU Instructions**: Smart contract resource usage
- **Memory Usage**: Heap allocation tracking

## CI/CD Integration

Performance tests run automatically in GitHub Actions:

- **On Push/PR**: Full test suite on main/develop branches
- **Scheduled**: Daily regression testing at 2 AM UTC
- **Artifacts**: Test reports and metrics stored for analysis

## Thresholds and Alerts

### Performance Baselines

| Metric | Warning | Critical |
|--------|---------|----------|
| P95 Latency | 500ms | 2000ms |
| Error Rate | 5% | 20% |
| Throughput | < 50 req/sec | < 10 req/sec |

### Contract Limits

| Operation | CPU Limit | Memory Limit |
|-----------|-----------|--------------|
| create_trade | 25M instructions | 1MB |
| Full happy path | 75M instructions | 2MB |
| Dispute cycle | 50M instructions | 1.5MB |

## Troubleshooting

### Common Issues

1. **Artillery not found**: Install globally with `npm install -g artillery`
2. **Port conflicts**: Ensure ports 3000, 5432, 6379 are available
3. **Database connection**: Wait for PostgreSQL to be ready
4. **Contract tests fail**: Ensure Rust toolchain is installed

### Performance Regression

If tests fail due to performance regression:

1. Check recent code changes
2. Review database query performance
3. Analyze resource usage patterns
4. Update baselines if legitimate performance changes

## Contributing

When adding new performance tests:

1. Add test configuration to appropriate `.yml` file
2. Update thresholds in monitoring configuration
3. Add documentation to this README
4. Update CI/CD workflow if needed
5. Test locally before committing

## Reports

Test results are saved to `reports/performance/`:
- JSON reports with detailed metrics
- HTML coverage reports for contracts
- Grafana dashboard snapshots
- Regression analysis reports