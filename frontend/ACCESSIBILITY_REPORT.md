# StellarEscrow Accessibility Report

**Generated:** 2026-03-29T17:12:53.450Z  
**Standard:** WCAG 2.1 Level AA  
**Project:** @stellar-escrow/frontend

---

## WCAG 2.1 AA Criterion Coverage

| Criterion | Name | Level | Test Coverage |
|-----------|------|-------|---------------|
| 1.1.1 | Non-text Content | A | ✅ Automated |
| 1.3.1 | Info and Relationships | A | ✅ Automated |
| 1.3.2 | Meaningful Sequence | A | ✅ Automated |
| 1.4.1 | Use of Color | A | ✅ Automated |
| 1.4.3 | Contrast (Minimum) | **AA** | ✅ Automated |
| 1.4.4 | Resize Text | **AA** | 📋 Manual (CV-03) |
| 1.4.10 | Reflow | **AA** | 📋 Manual (CV-04) |
| 1.4.11 | Non-text Contrast | **AA** | ✅ Automated |
| 1.4.12 | Text Spacing | **AA** | 📋 Manual |
| 1.4.13 | Content on Hover or Focus | **AA** | 📋 Manual |
| 2.1.1 | Keyboard | A | ✅ Automated |
| 2.1.2 | No Keyboard Trap | A | ✅ Automated |
| 2.3.3 | Animation from Interactions | AAA | ✅ Automated |
| 2.4.1 | Bypass Blocks | A | ✅ Automated |
| 2.4.3 | Focus Order | A | ✅ Automated |
| 2.4.4 | Link Purpose (In Context) | A | ✅ Automated |
| 2.4.6 | Headings and Labels | **AA** | ✅ Automated |
| 2.4.7 | Focus Visible | **AA** | ✅ Automated |
| 3.1.1 | Language of Page | A | ✅ Automated |
| 3.1.2 | Language of Parts | **AA** | ✅ Automated |
| 3.2.1 | On Focus | A | ✅ Automated |
| 3.2.2 | On Input | A | ✅ Automated |
| 3.3.1 | Error Identification | A | 📋 Manual (FE-01) |
| 3.3.2 | Labels or Instructions | A | ✅ Automated |
| 4.1.1 | Parsing | A | ✅ Automated |
| 4.1.2 | Name, Role, Value | A | ✅ Automated |
| 4.1.3 | Status Messages | **AA** | ✅ Automated |

---

## Automated Test Suites

| Suite | File | Description |
|-------|------|-------------|
| ARIA & Landmarks | `tests/aria.test.js` | Landmark structure, ARIA attributes, roles, modal accessibility |
| Keyboard Navigation | `tests/keyboard.test.js` | Tab order, Escape/Arrow keys, mobile menu, form submission |
| Colour Contrast | `tests/contrast.test.js` | Contrast ratios, CSS tokens, visual helpers |
| Screen Reader | `tests/screen-reader.test.js` | Live regions, dynamic content, status updates |
| i18n & RTL | `tests/i18n.test.js` | Translation lookup, RTL DOM, locale persistence |

Run all automated tests:
```bash
npm test
```

Run with coverage:
```bash
npm run test:coverage
```

---

## Manual Testing Procedures

## Screen Reader Testing

### SR-01 — Navigate the dashboard using NVDA (Windows) or VoiceOver (Mac/iOS)

**WCAG:** WCAG 1.3.1  
**Status:** PENDING

**Steps:**
1. Open index.html in Chrome/Firefox with NVDA enabled
2. Tab through all interactive elements — verify every control is announced
3. Use heading navigation (H key in NVDA) to jump between sections
4. Verify landmarks are announced (Navigation, Main, Banner, ContentInfo)
5. Confirm live regions announce WebSocket status changes and toast messages

**Expected result:** All content, structure, and state changes are announced correctly


### SR-02 — Verify wallet connect flow is operable via screen reader

**WCAG:** WCAG 4.1.2  
**Status:** PENDING

**Steps:**
1. Navigate to "Connect Wallet" button using Tab
2. Activate with Enter/Space — verify menu opens and is announced
3. Navigate wallet options with Tab/Arrow keys
4. Select "Freighter" — verify connection status is announced
5. Verify "Wallet disconnected" is announced on disconnect

**Expected result:** All wallet interactions are accessible and announced


### SR-03 — Dispute modal is accessible via screen reader

**WCAG:** WCAG 1.3.1  
**Status:** PENDING

**Steps:**
1. Click "Raise Dispute" — verify focus moves to modal
2. Verify modal title is announced as dialog label
3. Complete form fields — verify labels and hints are read
4. Submit and verify error/success announcements
5. Press Escape — verify modal closes and focus returns to trigger

**Expected result:** Modal is announced as a dialog; all content is reachable


### SR-04 — Arabic (RTL) mode is usable with screen reader

**WCAG:** WCAG 3.1.2  
**Status:** PENDING

**Steps:**
1. Switch language to Arabic using the language select
2. Verify html[lang] changes to "ar" (inspect element)
3. Verify reading order matches visual RTL layout
4. Confirm form labels still precede their inputs in reading order

**Expected result:** RTL content is read in correct logical order

---
## Keyboard Navigation

### KB-01 — Complete all primary user flows using keyboard only

**WCAG:** WCAG 2.1.1  
**Status:** PENDING

**Steps:**
1. Use Tab/Shift+Tab to navigate the full page without a mouse
2. Filter events using only the keyboard (Tab to form, change values, Enter to submit)
3. Open and close the mobile menu via keyboard on a narrow viewport
4. Open help center tabs using Enter/Space
5. Navigate FAQ accordions with keyboard

**Expected result:** Every feature is fully operable without a mouse


### KB-02 — Focus indicator is visible on all interactive elements

**WCAG:** WCAG 2.4.7  
**Status:** PENDING

**Steps:**
1. Tab through every interactive element
2. Verify a visible focus ring appears on each focused element
3. Test in both default and high-contrast modes
4. Check focus rings on: buttons, links, inputs, selects, textareas, the events log div

**Expected result:** Every focused element has a visible 3px solid outline


### KB-03 — No keyboard traps exist

**WCAG:** WCAG 2.1.2  
**Status:** PENDING

**Steps:**
1. Open the dispute modal with keyboard
2. Tab through all modal controls
3. Verify Tab wraps within the modal (focus trap)
4. Press Escape — verify modal closes and focus returns
5. Verify main page content is not Tab-accessible while modal is open

**Expected result:** Focus is trapped inside modal; Escape releases the trap


### KB-04 — Skip link functions correctly

**WCAG:** WCAG 2.4.1  
**Status:** PENDING

**Steps:**
1. Load the page and immediately press Tab
2. Verify skip link appears visually (top-centre of viewport)
3. Press Enter — verify viewport scrolls to main content
4. Verify focus lands on #main-content

**Expected result:** Skip link is visible on focus and moves focus to main content

---
## Colour & Visual

### CV-01 — Verify text contrast ratios meet AA with browser DevTools

**WCAG:** WCAG 1.4.3  
**Status:** PENDING

**Steps:**
1. Open Chrome DevTools → Elements → select body text elements
2. Use the Accessibility panel to verify contrast ratios
3. Check: primary text on card backgrounds (expect ≥4.5:1)
4. Check: secondary text on card backgrounds (expect ≥4.5:1)
5. Check: white text on accent/button backgrounds (expect ≥3.0:1 for large text)

**Expected result:** All text meets WCAG AA contrast thresholds


### CV-02 — UI component contrast meets 3:1

**WCAG:** WCAG 1.4.11  
**Status:** PENDING

**Steps:**
1. Check focus ring against adjacent background colours
2. Check form input borders against input background
3. Check status indicator dots against card background
4. Check toggle button border against page background

**Expected result:** All UI components have ≥3:1 contrast against adjacent colours


### CV-03 — Text resizes to 200% without loss of content

**WCAG:** WCAG 1.4.4  
**Status:** PENDING

**Steps:**
1. Set browser text size to 200% (browser zoom or Ctrl/Cmd + zoom)
2. Verify all text is still readable and not clipped
3. Verify layout does not break at 200%
4. Check nav, cards, tables, modals at 200%

**Expected result:** Page is readable and functional at 200% text size


### CV-04 — Page reflows correctly on narrow viewports (320px)

**WCAG:** WCAG 1.4.10  
**Status:** PENDING

**Steps:**
1. Set browser width to 320px (DevTools responsive mode)
2. Verify no horizontal scrollbar appears
3. Verify all content is accessible without horizontal scrolling
4. Check that tables, filters, and modals adapt to narrow width

**Expected result:** Content reflows with no horizontal scrolling at 320px

---
## Motion & Animation

### MO-01 — Reduced motion preference disables animations

**WCAG:** WCAG 2.3.3  
**Status:** PENDING

**Steps:**
1. Enable "Reduce Motion" in OS accessibility settings (or DevTools)
2. Reload the page — verify no animations play on load
3. Add events to the feed — verify no slide/fade animations
4. Open/close modals — verify no transition animations
5. Open toast notifications — verify instant appearance

**Expected result:** All animations are disabled when prefers-reduced-motion is active

---
## Forms & Error Handling

### FE-01 — Form validation errors are identified and described

**WCAG:** WCAG 3.3.1  
**Status:** PENDING

**Steps:**
1. Submit the dispute form empty — verify error messages appear
2. Verify error messages are associated with their inputs
3. Verify screen reader announces validation errors
4. Verify invalid inputs are marked with aria-invalid="true"
5. Correct errors and re-submit — verify success message

**Expected result:** Errors are programmatically identified and described


### FE-02 — Labels and instructions are provided before inputs

**WCAG:** WCAG 3.3.2  
**Status:** PENDING

**Steps:**
1. Verify every form field has a visible label above/beside it
2. Verify helper text (form-help spans) are linked via aria-describedby
3. Verify required fields are indicated visually (asterisk) and programmatically (required attribute)

**Expected result:** All inputs have labels and appropriate instructions


---

## Screen Reader Testing Matrix

| Screen Reader | Browser | OS | Priority |
|--------------|---------|-----|----------|
| NVDA (latest) | Chrome | Windows | High |
| NVDA (latest) | Firefox | Windows | High |
| VoiceOver | Safari | macOS | High |
| VoiceOver | Safari | iOS | High |
| TalkBack | Chrome | Android | Medium |
| JAWS | Chrome | Windows | Medium |

### Testing Protocol per Screen Reader

1. **Landmark navigation** — verify all regions are announced
2. **Heading navigation** — jump between h1→h2→h3 correctly
3. **Form interaction** — all labels, hints, and errors announced
4. **Live regions** — WebSocket status, toast notifications, search results
5. **Modals** — dialog announced; focus trapped; Escape works
6. **RTL mode** — Arabic content read in correct order

---

## Accessibility Checklist (pre-release)

### HTML
- [ ] All images have `alt` attributes
- [ ] All form inputs have associated `<label>` or `aria-label`
- [ ] All interactive elements are reachable via Tab
- [ ] No positive `tabindex` values
- [ ] Skip link present and functional
- [ ] Correct heading hierarchy (no skipped levels)
- [ ] All `<section>` elements have accessible names
- [ ] All `<nav>` elements have `aria-label`
- [ ] Modals have `role="dialog"` + `aria-modal="true"` + `aria-labelledby`
- [ ] Tables have `<caption>` or `aria-describedby`
- [ ] Table headers use `<th scope="col|row">`

### ARIA
- [ ] Live regions present for dynamic content
- [ ] `aria-expanded` updated on menus/dropdowns
- [ ] `aria-pressed` used on toggle buttons
- [ ] `aria-current="page"` on active nav link
- [ ] `aria-invalid` set on errored inputs

### CSS / Visual
- [ ] `:focus-visible` styles present
- [ ] Focus ring has ≥3px outline
- [ ] High-contrast mode (`[data-theme="high-contrast"]`) functional
- [ ] `prefers-reduced-motion` media query disables all animations
- [ ] `.visually-hidden` class used for screen-reader-only text
- [ ] Colour is not the sole means of conveying information
- [ ] Print styles hide non-essential UI elements

### JavaScript
- [ ] `announce()` function used for all dynamic state changes
- [ ] Toast notifications have `role="alert"`
- [ ] Escape key closes open menus/modals
- [ ] Arrow keys navigate within composite widgets (tables, menus)
- [ ] Focus returns to trigger after modal closes
- [ ] Language change updates `html[lang]` and `dir`

---

## Known Limitations

1. **Automated contrast tests** use hex values from CSS tokens; browser-computed
   colours after theme inheritance are verified via manual DevTools audit.
2. **Focus trap** in modals is implemented in JavaScript; automated tests verify
   the correct DOM structure; manual KB-03 verifies runtime behaviour.
3. **Screen reader testing** requires human testers with assistive technology;
   automated tests verify the correct ARIA markup is present.

---

*Report generated by `src/report.js`. Update manual test statuses in the MANUAL_CHECKLIST object.*