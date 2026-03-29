# Testing Guidelines — StellarEscrow

**Version:** 1.0 | **Last Updated:** 2026-03-27

Practical guidelines for writing, organizing, and maintaining tests across
the StellarEscrow codebase.

---

## 1. General Principles

- **One assertion per test** where possible — makes failures obvious
- **Descriptive test names** — `it('rejects trade with zero amount')` not `it('test 3')`
- **Arrange-Act-Assert** structure in every test
- **No test interdependence** — each test must pass in isolation
- **No hardcoded secrets** — use env vars or test fixtures
- **No `console.log` in tests** — use `expect` assertions

---

## 2. Unit Test Guidelines

### Naming Convention
```ts
describe('[Unit] — [ComponentName]', () => {
  it('[does something] when [condition]', () => { ... });
});
```

### File Location
Co-locate test files with source:
```
src/
  validation/
    tradeSchema.ts
    tradeSchema.test.ts   ← same directory
```

### What to Test
- All exported functions
- All branches (if/else, switch cases)
- Error cases and edge cases
- Boundary values (0, max, empty string)

### What NOT to Test
- Private/internal implementation details
- Third-party library behavior
- Framework internals (React rendering engine)

### Mocking
```ts
// Mock external dependencies, not the unit under test
jest.mock('../api/client');
const mockGet = jest.fn().mockResolvedValue({ data: [] });
```

---

## 3. Integration Test Guidelines (React Testing Library)

### Query Priority (highest to lowest)
1. `getByRole` — most accessible, preferred
2. `getByLabelText` — for form inputs
3. `getByPlaceholderText` — fallback for inputs
4. `getByText` — for non-interactive elements
5. `getByTestId` — last resort, use `data-testid`

```tsx
// ✅ Good — queries by accessible role
const button = screen.getByRole('button', { name: 'Create Trade' });

// ❌ Avoid — queries by implementation detail
const button = container.querySelector('.btn-primary');
```

### User Interactions
```tsx
// ✅ Use userEvent for realistic interactions
import userEvent from '@testing-library/user-event';
await userEvent.type(screen.getByLabelText('Amount'), '100');
await userEvent.click(screen.getByRole('button', { name: 'Submit' }));

// ❌ Avoid fireEvent for user interactions
fireEvent.change(input, { target: { value: '100' } });
```

### Async Testing
```tsx
// ✅ Wait for async state changes
await waitFor(() => {
  expect(screen.getByText('Trade created')).toBeInTheDocument();
});

// ✅ Or use findBy* queries (auto-waits)
const status = await screen.findByText('Trade created');
```

---

## 4. E2E Test Guidelines (Cypress)

### Selector Strategy
```ts
// ✅ Use data-testid for stable selectors
cy.get('[data-testid="create-trade-btn"]').click();

// ✅ Or accessible queries via cypress-testing-library
cy.findByRole('button', { name: 'Create Trade' }).click();

// ❌ Avoid CSS class selectors (break on refactor)
cy.get('.btn-primary').click();
```

### Test Structure
```ts
describe('Trade Flow', () => {
  beforeEach(() => {
    cy.visit('/');
    cy.intercept('POST', '/api/trades', { fixture: 'trade.json' }).as('createTrade');
  });

  it('creates a trade successfully', () => {
    cy.get('[data-testid="create-trade-btn"]').click();
    cy.get('[data-testid="seller-address"]').type('G...');
    cy.get('[data-testid="submit-trade"]').click();
    cy.wait('@createTrade');
    cy.get('[data-testid="trade-status"]').should('contain', 'created');
  });
});
```

### Network Stubbing
- Stub all external API calls in E2E tests
- Use `cy.intercept()` to control responses
- Test both success and error responses

---

## 5. Rust/Contract Test Guidelines

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Setup helper — shared across tests
    fn setup() -> (Env, Address, StellarEscrowContractClient) { ... }

    #[test]
    fn test_create_trade_with_valid_params() { ... }

    #[test]
    fn test_create_trade_fails_for_zero_amount() { ... }
}
```

### Naming Convention
- `test_[function]_[condition]` for happy path
- `test_[function]_fails_for_[reason]` for error cases

### What to Test
- Every public contract function
- Every error variant in `errors.rs`
- State transitions (Created → Funded → Completed)
- Authorization checks (only admin can update fee)

---

## 6. Regression Test Guidelines

### When to Add a Regression Test
- After fixing a bug: add a test that would have caught it
- When adding a new API endpoint: add to `api-contracts.regression.ts`
- When changing a threshold or constant: add to the relevant regression suite

### Regression Test Naming
```ts
// Include the thing being locked in
it('high_error_rate alert threshold is 5%', () => { ... });
it('buyer cannot equal seller', () => { ... });
```

---

## 7. Coverage Guidelines

### Thresholds
| Package | Branches | Functions | Lines | Statements |
|---------|----------|-----------|-------|------------|
| components | 70% | 70% | 70% | 70% |
| api | 70% | 70% | 70% | 70% |
| frontend | 70% | 70% | 70% | 70% |

### Checking Coverage
```bash
npm run test:coverage          # all packages
cd components && npm run test:coverage  # single package
```

### Coverage Anti-Patterns
- Don't write tests just to hit coverage numbers
- Don't test trivial getters/setters
- Do test all branches of business logic

---

## 8. Test Data Best Practices

```ts
// ✅ Use constants for test addresses
const VALID_SELLER = 'GBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQP';
const VALID_BUYER  = 'GGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM';

// ✅ Use factory functions for complex objects
function createMockTrade(overrides = {}) {
  return { id: '1', seller: VALID_SELLER, buyer: VALID_BUYER, amount: '100', ...overrides };
}

// ❌ Don't use real addresses or amounts
const seller = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU'; // real address
```

---

## 9. CI/CD Integration

All tests run automatically on every push and PR. See:
- `.github/workflows/regression.yml` — regression suite
- `.github/workflows/uat.yml` — UAT scenarios
- `.github/workflows/accessibility.yml` — accessibility tests
- `.github/workflows/terraform.yml` — infrastructure tests

**Never merge with failing tests.** If a test is flaky, fix it — don't skip it.
