# Test Plan — [Feature Name]

**Version:** 1.0
**Author:** [Name]
**Date:** [YYYY-MM-DD]
**Feature Branch:** `feature/[name]`
**Jira/Issue:** [Link]

---

## 1. Objective

_One paragraph describing what this test plan covers and what "done" looks like._

---

## 2. Scope

### In Scope
- [ ] [Component/service being tested]
- [ ] [API endpoints affected]
- [ ] [UI flows affected]

### Out of Scope
- [ ] [What is explicitly not being tested and why]

---

## 3. Test Approach

| Type | Tool | Owner | Environment |
|------|------|-------|-------------|
| Unit | Jest / cargo test | Developer | Local + CI |
| Integration | RTL / Jest | Developer | CI |
| E2E | Cypress | QA | Staging |
| Accessibility | axe-core + manual | QA | CI + Manual |
| Performance | k6 / manual | QA | Staging |

---

## 4. Test Cases

### 4.1 Happy Path

| ID | Description | Steps | Expected Result | Priority |
|----|-------------|-------|-----------------|----------|
| TC-001 | [Description] | 1. [Step] 2. [Step] | [Expected] | High |
| TC-002 | | | | |

### 4.2 Error Cases

| ID | Description | Steps | Expected Result | Priority |
|----|-------------|-------|-----------------|----------|
| TC-010 | [Description] | 1. [Step] | [Expected] | High |

### 4.3 Edge Cases

| ID | Description | Steps | Expected Result | Priority |
|----|-------------|-------|-----------------|----------|
| TC-020 | [Description] | 1. [Step] | [Expected] | Medium |

### 4.4 Accessibility

| ID | WCAG Criterion | Description | Expected Result | Priority |
|----|---------------|-------------|-----------------|----------|
| A-001 | 2.1.1 | All new UI elements keyboard accessible | Tab reaches all elements | High |
| A-002 | 4.1.2 | New components have accessible names | Screen reader announces correctly | High |

---

## 5. Entry Criteria

- [ ] Feature implementation complete
- [ ] Unit tests written and passing
- [ ] Code reviewed and approved
- [ ] Test environment available and seeded

---

## 6. Exit Criteria

- [ ] All P0/P1 test cases pass
- [ ] Coverage ≥ 70% for new code
- [ ] No open P0/P1 defects
- [ ] Accessibility automated tests pass
- [ ] QA sign-off obtained

---

## 7. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| [Risk description] | High/Med/Low | High/Med/Low | [Mitigation] |

---

## 8. Test Schedule

| Activity | Owner | Start | End |
|----------|-------|-------|-----|
| Unit test implementation | Developer | | |
| Integration test implementation | Developer | | |
| E2E test implementation | QA | | |
| Manual accessibility testing | QA | | |
| Test execution | QA | | |
| Defect fix and retest | Developer | | |
| Sign-off | QA Lead | | |

---

## 9. Sign-Off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Developer | | | |
| QA Engineer | | | |
| Tech Lead | | | |
