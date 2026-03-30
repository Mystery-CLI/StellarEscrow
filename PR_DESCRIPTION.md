## Summary

- expands the Soroban smart contract suite with dedicated edge case, security, integration, performance, and stress coverage
- repairs the contract core so the smart contract crate builds and all contract tests pass again
- adds root test commands, contract testing docs, snapshot artifacts, and a GitHub Actions workflow for contract testing and coverage

## What Changed

- added shared contract test harnesses and new suites under `contract/tests/`
- restored a working contract implementation in `contract/src/` for escrow, bridge, insurance, pause, compliance, migration, and analytics flows
- enabled importing the contract crate in integration tests via `rlib`
- documented the new contract test matrix and commands
- added `.github/workflows/smart-contract-tests.yml`

## Verification

```bash
cargo test --manifest-path contract/Cargo.toml
```

All contract unit, edge, security, integration, performance, and stress tests pass locally.

## Notes

- stress scenarios were tuned to stable CI-safe volumes so the Soroban test host does not hit budget exhaustion during the suite
- contract snapshots were generated under `contract/test_snapshots/` during verification
