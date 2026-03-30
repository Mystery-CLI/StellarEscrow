# Smart Contract Test Matrix

The Soroban contract test suite is split so CI and local runs can target the right risk area quickly.

## Suites

| Suite | File | Purpose |
|---|---|---|
| Edge cases | `contract/tests/edge_cases.rs` | Boundary values, invalid transitions, expiry handling |
| Security | `contract/tests/security.rs` | Pause controls, unauthorized paths, registration guards |
| Integration | `contract/tests/integration.rs` | End-to-end lifecycle flows across escrow, bridge, and insurance |
| Performance | `contract/tests/performance.rs` | Cost-estimate benchmarks with explicit instruction ceilings |
| Stress | `contract/tests/stress.rs` | High-volume load, scalability, and resource-usage monitoring |

## Commands

```bash
npm run test:contract
npm run test:contract:edge
npm run test:contract:security
npm run test:contract:integration
npm run test:contract:performance
npm run test:contract:stress
npm run test:contract:coverage
```

## Coverage

Coverage is generated with `cargo llvm-cov` and written to:

- `contract/coverage/lcov.info`
- `contract/coverage/html/index.html`

## GitHub CI

The `smart-contract-tests.yml` workflow runs the functional suites, stress benchmarks, and coverage upload on pushes and pull requests to `main` and `develop`.
