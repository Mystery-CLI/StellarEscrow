# App Testing Guidelines

## Scope

The `app` workspace uses a three-layer testing strategy:

- Unit tests: pure functions and isolated logic (`src/test/unit`)
- Integration tests: React component behavior with user interaction (`src/test/integration`)
- E2E tests: real browser flows (`cypress/e2e`)

## Tooling

- Unit + integration: Jest + `ts-jest` + React Testing Library
- E2E: Cypress
- Coverage: Jest coverage reports (`text`, `lcov`, `html`)

## Commands

```bash
# From repository root
npm run test --workspace=app
npm run test:unit --workspace=app
npm run test:integration --workspace=app
npm run test:coverage --workspace=app
npm run test:e2e --workspace=app
npm run test:e2e:open --workspace=app
```

## Coverage Standard

- Minimum threshold: `70%` for branches, functions, lines, and statements.
- Coverage output path: `app/coverage/`.

## Authoring Rules

- Prefer `getByRole`, `getByLabelText`, and visible text over implementation-specific selectors.
- Use `userEvent` for realistic interaction in integration tests.
- Keep unit tests deterministic and free of network/browser side effects.
- Keep each Cypress spec focused on one user flow and assert user-visible outcomes.

## File Conventions

- Unit tests: `src/test/unit/**/*.test.ts?(x)`
- Integration tests: `src/test/integration/**/*.test.ts?(x)`
- E2E tests: `cypress/e2e/**/*.cy.ts`
