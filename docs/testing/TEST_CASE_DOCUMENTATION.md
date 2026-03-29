# Test Case Documentation — StellarEscrow

**Version:** 1.0 | **Last Updated:** 2026-03-27

This document catalogs the canonical test cases for core platform features.
Each test case maps to acceptance criteria and WCAG criteria where applicable.

---

## TC-TRADE: Trade Lifecycle

| ID | Description | Type | File | AC | Status |
|----|-------------|------|------|----|--------|
| TC-TRADE-001 | Create trade with valid seller, buyer, amount | Unit | `tradeSchema.test.ts` | AC-1 | ✅ |
| TC-TRADE-002 | Reject trade with identical buyer and seller | Unit | `tradeSchema.test.ts` | AC-1 | ✅ |
| TC-TRADE-003 | Reject trade with zero amount | Unit | `tradeSchema.test.ts` | AC-1 | ✅ |
| TC-TRADE-004 | Reject trade with amount > 1,000,000,000 | Unit | `tradeSchema.test.ts` | AC-1 | ✅ |
| TC-TRADE-005 | Reject invalid Stellar address format | Unit | `tradeSchema.test.ts` | AC-1 | ✅ |
| TC-TRADE-006 | Fund trade changes status to Funded | Contract | `test.rs` | AC-3 | ✅ |
| TC-TRADE-007 | Complete + confirm trade releases funds to seller | Contract | `test.rs` | AC-4 | ✅ |
| TC-TRADE-008 | Cancel trade returns status to Cancelled | Contract | `test.rs` | AC-3 | ✅ |
| TC-TRADE-009 | Dispute trade changes status to Disputed | Contract | `test.rs` | AC-3 | ✅ |
| TC-TRADE-010 | Resolve dispute releases funds to buyer | Contract | `test.rs` | AC-4 | ✅ |
| TC-TRADE-011 | Trade form renders all required fields | Integration | `TradeForm.test.tsx` | AC-1 | ✅ |
| TC-TRADE-012 | Trade form shows validation errors on empty submit | Integration | `TradeForm.test.tsx` | AC-1 | ✅ |
| TC-TRADE-013 | Trade form auto-saves draft to localStorage | Integration | `TradeForm.test.tsx` | AC-1 | ✅ |
| TC-TRADE-014 | Trade card displays truncated addresses | Integration | `TradeCard.test.tsx` | AC-2 | ✅ |
| TC-TRADE-015 | E2E: Create trade via UI | E2E | `trade-flow.cy.ts` | AC-1 | ✅ |

---

## TC-COMPLIANCE: KYC/AML

| ID | Description | Type | File | AC | Status |
|----|-------------|------|------|----|--------|
| TC-COMP-001 | Compliance check returns status for any address | UAT | `compliance-ux.uat.ts` | AC-5 | ✅ |
| TC-COMP-002 | Compliance report returns required fields | UAT | `compliance-ux.uat.ts` | AC-7 | ✅ |
| TC-COMP-003 | Blocked address has risk_score ≥ threshold | Regression | `compliance-rules.regression.ts` | AC-8 | ✅ |
| TC-COMP-004 | Workflow: pending → approved transition allowed | Regression | `compliance-rules.regression.ts` | AC-6 | ✅ |
| TC-COMP-005 | Workflow: approved → pending transition forbidden | Regression | `compliance-rules.regression.ts` | AC-6 | ✅ |
| TC-COMP-006 | Report aggregates status counts correctly | Regression | `compliance-rules.regression.ts` | AC-7 | ✅ |
| TC-COMP-007 | Unverified KYC blocks trade creation | Contract | `test.rs` | AC-8 | ✅ |
| TC-COMP-008 | Blocked jurisdiction prevents trade | Contract | `test.rs` | AC-8 | ✅ |

---

## TC-MONITORING: Alerts and Metrics

| ID | Description | Type | File | AC | Status |
|----|-------------|------|------|----|--------|
| TC-MON-001 | Dashboard returns health_status field | UAT | `monitoring-ux.uat.ts` | AC-9 | ✅ |
| TC-MON-002 | Alerts endpoint returns array | UAT | `monitoring-ux.uat.ts` | AC-10 | ✅ |
| TC-MON-003 | Prometheus metrics endpoint returns text/plain | UAT | `monitoring-ux.uat.ts` | AC-11 | ✅ |
| TC-MON-004 | Health endpoint responds within 500ms | UAT | `monitoring-ux.uat.ts` | AC-12 | ✅ |
| TC-MON-005 | high_error_rate alert threshold is 5% | Regression | `alert-rules.regression.ts` | — | ✅ |
| TC-MON-006 | aml_high_risk_spike threshold is 5 (critical) | Regression | `alert-rules.regression.ts` | — | ✅ |
| TC-MON-007 | Alert fires when metric exceeds threshold (gt) | Regression | `alert-rules.regression.ts` | — | ✅ |
| TC-MON-008 | Alert does not fire at exact threshold (gt) | Regression | `alert-rules.regression.ts` | — | ✅ |

---

## TC-A11Y: Accessibility

| ID | WCAG | Description | Type | File | Status |
|----|------|-------------|------|------|--------|
| TC-A11Y-001 | 4.1.2 | Button has accessible name | Automated | `wcag-components.a11y.tsx` | ✅ |
| TC-A11Y-002 | 1.3.1 | Input has visible label | Automated | `wcag-components.a11y.tsx` | ✅ |
| TC-A11Y-003 | 3.3.1 | Input error has aria-describedby | Automated | `wcag-components.a11y.tsx` | ✅ |
| TC-A11Y-004 | 2.4.1 | Skip link targets main content | Automated | `wcag-components.a11y.tsx` | ✅ |
| TC-A11Y-005 | 1.3.1 | Data table has headers and caption | Automated | `wcag-components.a11y.tsx` | ✅ |
| TC-A11Y-006 | 1.1.1 | Informative image has alt text | Automated | `wcag-components.a11y.tsx` | ✅ |
| TC-A11Y-007 | 2.1.1 | All interactive elements keyboard accessible | Manual | `MANUAL_TESTING_PROCEDURES.md` | — |
| TC-A11Y-008 | 2.1.2 | No keyboard trap | Manual | `MANUAL_TESTING_PROCEDURES.md` | — |
| TC-A11Y-009 | 1.4.3 | Text contrast ≥ 4.5:1 | Manual | `MANUAL_TESTING_PROCEDURES.md` | — |
| TC-A11Y-010 | 4.1.3 | Screen reader announces live regions | Manual | `MANUAL_TESTING_PROCEDURES.md` | — |

---

## TC-API: API Contracts

| ID | Description | Type | File | Status |
|----|-------------|------|------|--------|
| TC-API-001 | GET /health returns 200 with status field | Regression | `api-contracts.regression.ts` | ✅ |
| TC-API-002 | GET /events returns paginated response | Regression | `api-contracts.regression.ts` | ✅ |
| TC-API-003 | GET /events respects limit parameter | Regression | `api-contracts.regression.ts` | ✅ |
| TC-API-004 | GET /compliance/report returns required fields | Regression | `api-contracts.regression.ts` | ✅ |
| TC-API-005 | GET /monitoring/dashboard returns health_status | Regression | `api-contracts.regression.ts` | ✅ |
| TC-API-006 | GET /metrics returns Prometheus text format | Regression | `api-contracts.regression.ts` | ✅ |
