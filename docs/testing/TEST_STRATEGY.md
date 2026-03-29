# StellarEscrow — Test Strategy

**Version:** 1.0 | **Owner:** Engineering | **Last Updated:** 2026-03-27

---

## 1. Purpose

This document defines the overall testing strategy for the StellarEscrow platform.
It establishes the testing philosophy, scope, tools, environments, and quality gates
that govern all testing activities across the platform.

---

## 2. Testing Philosophy

- **Shift left**: catch defects as early as possible — in unit tests, not production
- **Test behavior, not implementation**: tests should survive refactors
- **Automate everything repeatable**: manual testing is reserved for exploratory and accessibility work
- **Fail fast**: CI must block merges on test failures
- **Coverage is a floor, not a ceiling**: 70% minimum, aim for 85%+

---

## 3. Scope

### In Scope
- Smart contract logic (Rust/Soroban)
- Indexer service (Rust)
- API integration layer (TypeScript)
- React component library
- Frontend JavaScript
- Compliance, monitoring, analytics, webhook services
- Infrastructure (Terraform plan validation)
- Accessibility (WCAG 2.1 AA)

### Out of Scope
- Third-party service internals (Stellar network, AWS)
- Browser rendering engines
- Operating system behavior

---

## 4. Testing Pyramid

```
         ┌─────────────────┐
         │   E2E / UAT     │  ← Cypress, manual UAT
         │   (few, slow)   │
         ├─────────────────┤
         │  Integration    │  ← React Testing Library, API contract tests
         │  (some, medium) │
         ├─────────────────┤
         │   Unit Tests    │  ← Jest, cargo test
         │  (many, fast)   │
         └─────────────────┘
```

| Layer | Count Target | Speed | Tools |
|-------|-------------|-------|-------|
| Unit | 70% of tests | < 5ms each | Jest, cargo test |
| Integration | 20% of tests | < 500ms each | RTL, Jest |
| E2E | 10% of tests | < 30s each | Cypress |
| Regression | Automated | < 2min total | Jest (regression suite) |
| UAT | Per release | Manual + automated | Jest (UAT suite) |
| Accessibility | Per PR | < 1min | axe-core, manual |

---

## 5. Test Environments

| Environment | Purpose | Data | Trigger |
|-------------|---------|------|---------|
| Local | Developer testing | Mocked/seeded | On save (watch mode) |
| CI | Automated gate | Seeded test DB | Every push/PR |
| Staging | Integration testing | Anonymized prod-like | On merge to develop |
| Production | Smoke tests only | Live | Post-deploy |

---

## 6. Quality Gates

### Pull Request Gate (must pass to merge)
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Coverage ≥ 70% (no regression from baseline)
- [ ] Regression suite passes (no previously-passing tests now failing)
- [ ] Accessibility automated tests pass
- [ ] No new lint errors

### Release Gate (must pass before production deploy)
- [ ] All CI gates pass
- [ ] UAT scenarios pass (≥ 90% pass rate)
- [ ] Accessibility manual checklist signed off
- [ ] Performance benchmarks within SLA
- [ ] Security scan clean

---

## 7. Defect Classification

| Severity | Definition | SLA |
|----------|-----------|-----|
| P0 Critical | Data loss, security breach, complete outage | Fix within 4 hours |
| P1 High | Core feature broken, significant user impact | Fix within 24 hours |
| P2 Medium | Feature degraded, workaround exists | Fix within 1 sprint |
| P3 Low | Minor issue, cosmetic | Fix within 2 sprints |

---

## 8. Test Data Management

- Unit tests: inline fixtures or factory functions
- Integration tests: seeded test database (reset between runs)
- E2E tests: dedicated test accounts with known state
- No real user data in any test environment
- PII replaced with generated test data

---

## 9. Metrics and Reporting

Track and report weekly:
- Overall pass rate per suite
- Coverage trend (must not decrease)
- Flaky test count (target: 0)
- Mean time to detect (MTTD) — time from bug introduction to test failure
- Accessibility violation count trend

---

## 10. Roles and Responsibilities

| Role | Responsibility |
|------|---------------|
| Developer | Write unit + integration tests for all new code |
| QA Engineer | Write E2E tests, maintain regression suite, run manual tests |
| Accessibility Lead | Maintain WCAG checklist, run screen reader tests |
| Tech Lead | Review test strategy, approve quality gate changes |
| Product Owner | Sign off UAT scenarios |
