# Requirements Document

## Introduction

This feature establishes comprehensive unit testing for all React components and validation utilities in the `components/` package. The library contains base UI components (Alert, Badge, Button, Card, Input), escrow-domain components (EventFeed, TradeCard, TradeForm, TradeStatus), and a trade validation schema. Several test files already exist; this spec defines the complete, consistent coverage standard that all components must meet, including rendering, interaction, accessibility, snapshot, and coverage-threshold requirements.

## Glossary

- **Test_Suite**: The Jest + React Testing Library test runner configured in `components/jest.config.js`
- **Component**: A React functional component exported from `components/src/`
- **Utility**: A non-component TypeScript module exported from `components/src/validation/`
- **Snapshot**: A serialised rendering of a component stored by Jest for regression detection
- **Coverage_Report**: The output of `jest --coverage` measuring statements, branches, functions, and lines
- **Accessibility_Check**: An assertion that a rendered component exposes correct ARIA roles, labels, and attributes
- **TradeForm**: The `TradeForm` component in `components/src/escrow/TradeForm.tsx`
- **TradeCard**: The `TradeCard` component in `components/src/escrow/TradeCard.tsx`
- **TradeStatus**: The `TradeStatus` component in `components/src/escrow/TradeStatus.tsx`
- **EventFeed**: The `EventFeed` component in `components/src/escrow/EventFeed.tsx`
- **Alert**: The `Alert` component in `components/src/base/Alert.tsx`
- **Badge**: The `Badge` component in `components/src/base/Badge.tsx`
- **Button**: The `Button` component in `components/src/base/Button.tsx`
- **Card**: The `Card` component in `components/src/base/Card.tsx`
- **Input**: The `Input` component in `components/src/base/Input.tsx`
- **TradeSchema**: The validation utilities exported from `components/src/validation/tradeSchema.ts`
- **Draft**: Form state persisted to `localStorage` by TradeForm's auto-save feature

---

## Requirements

### Requirement 1: Base Component Test Coverage

**User Story:** As a developer, I want every base UI component to have a dedicated test file, so that regressions in shared primitives are caught immediately.

#### Acceptance Criteria

1. THE Test_Suite SHALL include a test file for each of the five base components: Alert, Badge, Button, Card, and Input.
2. WHEN a base component is rendered with its default props, THE Test_Suite SHALL assert that the component mounts without throwing an error.
3. WHEN a base component receives a variant or type prop, THE Test_Suite SHALL assert that the corresponding CSS class is applied to the root element.
4. WHEN the Alert component receives an `onClose` callback prop, THE Test_Suite SHALL assert that clicking the close button invokes the callback exactly once.
5. WHEN the Alert component is rendered without an `onClose` prop, THE Test_Suite SHALL assert that no close button is present in the DOM.
6. WHEN the Card component receives a `title` prop, THE Test_Suite SHALL assert that the title text is rendered inside the card.
7. WHEN the Card component is rendered without a `title` prop, THE Test_Suite SHALL assert that no heading element is rendered inside the card.
8. WHEN the Badge component receives a `variant` prop, THE Test_Suite SHALL assert that the rendered element carries the class `badge-{variant}`.

---

### Requirement 2: Escrow Component Test Coverage

**User Story:** As a developer, I want every escrow-domain component to have a dedicated test file, so that trade-workflow regressions are detected before release.

#### Acceptance Criteria

1. THE Test_Suite SHALL include a test file for each of the four escrow components: EventFeed, TradeCard, TradeForm, and TradeStatus.
2. WHEN TradeStatus is rendered with each of the five valid status values (`created`, `funded`, `completed`, `disputed`, `cancelled`), THE Test_Suite SHALL assert that the correct human-readable label is displayed.
3. WHEN TradeStatus is rendered with each valid status, THE Test_Suite SHALL assert that the Badge variant matches the mapping defined in `statusConfig`.
4. WHEN EventFeed receives an empty `events` array, THE Test_Suite SHALL assert that the empty-state message "No events yet" is rendered.
5. WHEN EventFeed receives a non-empty `events` array, THE Test_Suite SHALL assert that one list item is rendered per event.
6. WHEN an event item in EventFeed is clicked and an `onEventClick` callback is provided, THE Test_Suite SHALL assert that the callback is invoked with the corresponding event object.
7. WHEN EventFeed is rendered, THE Test_Suite SHALL assert that the container element carries `role="log"` and an accessible label.

---

### Requirement 3: TradeForm Interaction and Validation Tests

**User Story:** As a developer, I want TradeForm interactions and validation logic to be fully tested, so that form correctness is guaranteed across all user input paths.

#### Acceptance Criteria

1. WHEN TradeForm is submitted with all fields empty, THE Test_Suite SHALL assert that error messages for `seller`, `buyer`, and `amount` are displayed and `onSubmit` is not called.
2. WHEN TradeForm is submitted with valid values for all required fields, THE Test_Suite SHALL assert that `onSubmit` is called with the correct field values.
3. WHEN a required field in TradeForm loses focus while empty, THE Test_Suite SHALL assert that the corresponding error message appears.
4. WHEN a field in TradeForm transitions from invalid to valid, THE Test_Suite SHALL assert that the error message is removed.
5. WHEN the `buyer` field value equals the `seller` field value, THE Test_Suite SHALL assert that a cross-field validation error is displayed.
6. WHEN TradeForm receives a `loading` prop set to `true`, THE Test_Suite SHALL assert that the submit button is disabled.
7. WHEN TradeForm auto-save is enabled and a field value changes, THE Test_Suite SHALL assert that the Draft is written to `localStorage` after the debounce interval elapses.
8. WHEN TradeForm mounts and a Draft exists in `localStorage`, THE Test_Suite SHALL assert that the form fields are pre-populated with the Draft values.
9. WHEN TradeForm is successfully submitted, THE Test_Suite SHALL assert that the Draft is removed from `localStorage`.

---

### Requirement 4: Validation Utility Tests

**User Story:** As a developer, I want the TradeSchema validation utilities to be thoroughly tested, so that invalid trade data is reliably rejected at the boundary.

#### Acceptance Criteria

1. THE Test_Suite SHALL include tests for `isValidStellarAddress`, `isPositiveNumber`, `validateField`, `isFieldValid`, and `validateTradeForm`.
2. WHEN `isValidStellarAddress` is called with a 56-character string starting with `G` containing only uppercase base-32 characters, THE Test_Suite SHALL assert that the function returns `true`.
3. WHEN `isValidStellarAddress` is called with a string that does not start with `G`, is shorter than 56 characters, or contains lowercase letters, THE Test_Suite SHALL assert that the function returns `false`.
4. WHEN `isPositiveNumber` is called with a string representing a number greater than zero, THE Test_Suite SHALL assert that the function returns `true`.
5. WHEN `isPositiveNumber` is called with zero, a negative number, a non-numeric string, or an empty string, THE Test_Suite SHALL assert that the function returns `false`.
6. WHEN `validateTradeForm` is called with a fully valid form object, THE Test_Suite SHALL assert that the returned errors object contains zero keys.
7. WHEN `validateTradeForm` is called with a form object where `buyer` equals `seller`, THE Test_Suite SHALL assert that the returned errors object contains a `buyer` key.
8. FOR ALL valid form objects `f`, calling `validateTradeForm(f)` twice SHALL produce an equivalent result (idempotence property).

---

### Requirement 5: Snapshot Testing

**User Story:** As a developer, I want snapshot tests for each component's default rendering, so that unintended visual regressions are surfaced during code review.

#### Acceptance Criteria

1. THE Test_Suite SHALL include at least one snapshot test per component (Alert, Badge, Button, Card, Input, EventFeed, TradeCard, TradeForm, TradeStatus).
2. WHEN a component is rendered with its default or representative props and a snapshot does not yet exist, THE Test_Suite SHALL create a new snapshot file.
3. WHEN a component's rendered output changes and an existing snapshot exists, THE Test_Suite SHALL fail and require an explicit snapshot update.
4. WHERE a component accepts multiple significant visual variants (e.g., Button `variant`, Alert `type`), THE Test_Suite SHALL include one snapshot per variant.

---

### Requirement 6: Accessibility Testing

**User Story:** As a developer, I want accessibility assertions in every component test file, so that ARIA compliance issues are caught before components reach production.

#### Acceptance Criteria

1. THE Test_Suite SHALL assert that every interactive element (buttons, inputs) has an accessible name resolvable by `getByRole` or `getByLabelText`.
2. WHEN the Input component is rendered with an `error` prop, THE Test_Suite SHALL assert that the input element carries `aria-invalid="true"`.
3. WHEN the Input component is rendered with an `error` or `helperText` prop, THE Test_Suite SHALL assert that the input's `aria-describedby` attribute references the visible message element.
4. WHEN the Alert component is rendered, THE Test_Suite SHALL assert that the container element carries `role="alert"`.
5. WHEN the Alert close button is rendered, THE Test_Suite SHALL assert that the button has an `aria-label` attribute.
6. WHEN EventFeed is rendered, THE Test_Suite SHALL assert that the container carries `role="log"` and a non-empty `aria-label`.
7. WHEN TradeForm is rendered, THE Test_Suite SHALL assert that every form field has a programmatically associated label (via `htmlFor`/`id` pairing).

---

### Requirement 7: Coverage Thresholds

**User Story:** As a developer, I want enforced coverage thresholds, so that the test suite cannot regress below a minimum quality bar.

#### Acceptance Criteria

1. THE Test_Suite SHALL enforce a minimum of 80% statement coverage across all files in `components/src/`.
2. THE Test_Suite SHALL enforce a minimum of 80% branch coverage across all files in `components/src/`.
3. THE Test_Suite SHALL enforce a minimum of 80% function coverage across all files in `components/src/`.
4. THE Test_Suite SHALL enforce a minimum of 80% line coverage across all files in `components/src/`.
5. IF the coverage for any metric falls below 80%, THEN THE Test_Suite SHALL exit with a non-zero status code, causing CI to fail.
6. THE Test_Suite SHALL exclude `*.stories.tsx` and `*.d.ts` files from coverage collection.
