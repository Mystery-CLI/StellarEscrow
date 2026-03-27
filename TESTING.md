# Testing Guidelines

## Overview

This project uses a three-tier testing strategy:

| Layer | Tool | Location | Purpose |
|-------|------|----------|---------|
| Unit | Jest + ts-jest | `*/src/**/*.test.ts(x)` | Functions, hooks, slices |
| Integration | Jest + React Testing Library / MSW | `components/src/**/*.test.tsx`, `api/src/**/*.integration.test.ts` | Component and API behaviour |
| Contract | Jest + runtime validators | `api/src/**/*.contract.test.ts` | Endpoint method/path/shape guarantees |
| Load | Jest + concurrent request harness | `api/src/**/*.load.test.ts` | API concurrency smoke coverage |
| Security | Jest + scenario harness | `security/src/security.assessment.test.ts` | Penetration tests, vulnerability scans, compliance, monitoring |
| E2E | Cypress | `components/cypress/e2e/` | Full user flows |

## Running Tests

```bash
# All unit/integration tests across all packages
npm test

# With coverage reports
npm run test:coverage

# API package suites
npm run test:integration --workspace=api
npm run test:contract --workspace=api
npm run test:load --workspace=api
npm run test:docs --workspace=api

# Security assessment suite
npm run test:security
npm run test:security --workspace=security

# E2E tests (requires app running on localhost:3000)
npm run test:e2e

# Watch mode (inside a package)
cd components && npm run test:watch
```

## Coverage Thresholds

All packages enforce **70% minimum** on branches, functions, lines, and statements.
Coverage reports are written to `coverage/` in each package directory.

## Unit Tests

- One test file per source file, co-located: `Button.tsx` → `Button.test.tsx`
- Use `describe` to group related cases; use `it`/`test` for individual assertions
- Mock external dependencies with `jest.fn()` or `jest.mock()`
- Keep tests independent — no shared mutable state between tests

```ts
// Good
it('disables button when loading', () => {
  render(<Button loading>Save</Button>);
  expect(screen.getByText('Save')).toBeDisabled();
});
```

## Integration Tests (React Testing Library)

- Test behaviour, not implementation details
- Query by accessible roles/labels (`getByRole`, `getByLabelText`) over test IDs
- Use `userEvent` over `fireEvent` for realistic interactions
- Assert on what the user sees, not internal state

```ts
import userEvent from '@testing-library/user-event';

it('submits form with valid data', async () => {
  const onSubmit = jest.fn();
  render(<TradeForm onSubmit={onSubmit} />);
  await userEvent.type(screen.getByLabelText('Amount (USDC)'), '100');
  await userEvent.click(screen.getByRole('button', { name: 'Create Trade' }));
  expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ amount: '100' }));
});
```

## E2E Tests (Cypress)

- Located in `components/cypress/e2e/`
- Use `data-testid` attributes for selectors that are stable across refactors
- Each spec file covers one user flow (e.g., `trade-flow.cy.ts`, `dispute-flow.cy.ts`)
- Run against a locally running app: `npm run test:e2e:open` for interactive mode

```ts
it('creates a new trade', () => {
  cy.visit('/');
  cy.get('[data-testid="create-trade-btn"]').click();
  cy.get('[data-testid="amount"]').type('100');
  cy.get('[data-testid="submit-trade"]').click();
  cy.get('[data-testid="trade-status"]').should('contain', 'created');
});
```

## What to Test

**Do test:**
- Happy path and error/edge cases for business logic
- Component rendering with different prop combinations
- Form validation and submission
- Redux slice reducers and selectors
- Security utilities (sanitization, validation)
- Security attack scenarios, compliance controls, vulnerability baselines, and monitoring alerts

**Don't test:**
- Third-party library internals
- Implementation details (internal state, private methods)
- Styling/CSS classes unless they affect behaviour

## Adding New Tests

1. Create `<ComponentName>.test.tsx` alongside the source file
2. Import from `@testing-library/react` for React components
3. Run `npm run test:coverage` to verify thresholds are met
4. For new user flows, add a Cypress spec in `components/cypress/e2e/`


---

## Regression Testing Suite

Located in `testing/regression/`. Catches breaking changes automatically on every PR and nightly.

### Suites

| Suite | File | What it tests |
|-------|------|---------------|
| API Contracts | `api-contracts.regression.ts` | All public endpoints — shape, status codes, required fields |
| Validation Schema | `validation-schema.regression.ts` | Trade form validation rules (pure functions, no network) |
| Compliance Rules | `compliance-rules.regression.ts` | KYC/AML workflow state machine and report aggregation |
| Alert Rules | `alert-rules.regression.ts` | Monitoring alert thresholds and evaluation logic |

### Running

```bash
# Run all regression suites
npm run test:regression

# Generate HTML + JSON report
npm run test:regression:report

# Analyze trends (flaky tests, pass rate trend)
npm run test:regression:analyze

# Detect regressions vs saved baseline
cd testing/regression && node scripts/detect-changes.js
```

### Change Detection

On first run, `detect-changes.js` saves a baseline. On subsequent runs it diffs against it and exits non-zero if any previously-passing test now fails. The baseline is stored in `testing/regression/reports/baseline.json`.

### CI

The `regression.yml` workflow runs on every push/PR:
- `unit-regression` job: validation, compliance, alert rule suites (no server needed)
- `api-regression` job: API contract suite (spins up indexer + postgres)
- Posts a pass/fail summary as a PR comment

---

## UAT Framework

Located in `testing/uat/`. Stakeholder-facing acceptance testing tied to numbered Acceptance Criteria.

### Scenarios

| Scenario | File | Acceptance Criteria |
|----------|------|---------------------|
| Trade Lifecycle | `trade-lifecycle.uat.ts` | AC-1 through AC-4 |
| Compliance UX | `compliance-ux.uat.ts` | AC-5 through AC-8 |
| Monitoring UX | `monitoring-ux.uat.ts` | AC-9 through AC-12 |

### Running

```bash
# Run all UAT scenarios
npm run test:uat

# Generate stakeholder HTML report (with sign-off table)
npm run test:uat:report

# Full automated pipeline (run + report + feedback + webhook)
npm run test:uat:automate

# With specific environment and Slack webhook
cd testing/uat && node scripts/run-automated-uat.js --env staging --webhook https://hooks.slack.com/...
```

### Feedback Collection

Stakeholders submit feedback via `testing/uat/reports/feedback-input.json`:

```json
{
  "stakeholder": "Product Owner",
  "name": "[name]",
  "overall_decision": "accept",
  "items": [
    { "ac": "AC-1", "decision": "accept", "notes": "Works as expected" }
  ],
  "general_notes": "Ready for production."
}
```

Then run: `cd testing/uat && node scripts/collect-feedback.js`

Feedback is appended to `feedback-log.json` and aggregated into `feedback-summary.json`.

### Reports

| File | Description |
|------|-------------|
| `uat-report.html` | Stakeholder HTML report with sign-off table |
| `uat-summary.json` | Machine-readable summary per AC |
| `feedback-log.json` | All stakeholder feedback entries |
| `feedback-summary.json` | Aggregated feedback by AC |

### CI

The `uat.yml` workflow runs on push to main/develop and supports manual dispatch with environment selection. Reports are retained for 90 days as artifacts.
