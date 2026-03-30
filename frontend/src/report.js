#!/usr/bin/env node
/**
 * StellarEscrow Accessibility Report Generator
 * Produces ACCESSIBILITY_REPORT.md combining automated results + manual checklists
 *
 * Usage: node src/report.js [--json path/to/jest-results.json]
 */

const fs = require('fs');
const path = require('path');

// ─── Manual Testing Checklist ─────────────────────────────────────────────────

const MANUAL_CHECKLIST = {
  'Screen Reader Testing': [
    {
      id: 'SR-01',
      criterion: 'WCAG 1.3.1',
      description: 'Navigate the dashboard using NVDA (Windows) or VoiceOver (Mac/iOS)',
      steps: [
        'Open index.html in Chrome/Firefox with NVDA enabled',
        'Tab through all interactive elements — verify every control is announced',
        'Use heading navigation (H key in NVDA) to jump between sections',
        'Verify landmarks are announced (Navigation, Main, Banner, ContentInfo)',
        'Confirm live regions announce WebSocket status changes and toast messages',
      ],
      expected: 'All content, structure, and state changes are announced correctly',
      status: 'PENDING',
    },
    {
      id: 'SR-02',
      criterion: 'WCAG 4.1.2',
      description: 'Verify wallet connect flow is operable via screen reader',
      steps: [
        'Navigate to "Connect Wallet" button using Tab',
        'Activate with Enter/Space — verify menu opens and is announced',
        'Navigate wallet options with Tab/Arrow keys',
        'Select "Freighter" — verify connection status is announced',
        'Verify "Wallet disconnected" is announced on disconnect',
      ],
      expected: 'All wallet interactions are accessible and announced',
      status: 'PENDING',
    },
    {
      id: 'SR-03',
      criterion: 'WCAG 1.3.1',
      description: 'Dispute modal is accessible via screen reader',
      steps: [
        'Click "Raise Dispute" — verify focus moves to modal',
        'Verify modal title is announced as dialog label',
        'Complete form fields — verify labels and hints are read',
        'Submit and verify error/success announcements',
        'Press Escape — verify modal closes and focus returns to trigger',
      ],
      expected: 'Modal is announced as a dialog; all content is reachable',
      status: 'PENDING',
    },
    {
      id: 'SR-04',
      criterion: 'WCAG 3.1.2',
      description: 'Arabic (RTL) mode is usable with screen reader',
      steps: [
        'Switch language to Arabic using the language select',
        'Verify html[lang] changes to "ar" (inspect element)',
        'Verify reading order matches visual RTL layout',
        'Confirm form labels still precede their inputs in reading order',
      ],
      expected: 'RTL content is read in correct logical order',
      status: 'PENDING',
    },
  ],

  'Keyboard Navigation': [
    {
      id: 'KB-01',
      criterion: 'WCAG 2.1.1',
      description: 'Complete all primary user flows using keyboard only',
      steps: [
        'Use Tab/Shift+Tab to navigate the full page without a mouse',
        'Filter events using only the keyboard (Tab to form, change values, Enter to submit)',
        'Open and close the mobile menu via keyboard on a narrow viewport',
        'Open help center tabs using Enter/Space',
        'Navigate FAQ accordions with keyboard',
      ],
      expected: 'Every feature is fully operable without a mouse',
      status: 'PENDING',
    },
    {
      id: 'KB-02',
      criterion: 'WCAG 2.4.7',
      description: 'Focus indicator is visible on all interactive elements',
      steps: [
        'Tab through every interactive element',
        'Verify a visible focus ring appears on each focused element',
        'Test in both default and high-contrast modes',
        'Check focus rings on: buttons, links, inputs, selects, textareas, the events log div',
      ],
      expected: 'Every focused element has a visible 3px solid outline',
      status: 'PENDING',
    },
    {
      id: 'KB-03',
      criterion: 'WCAG 2.1.2',
      description: 'No keyboard traps exist',
      steps: [
        'Open the dispute modal with keyboard',
        'Tab through all modal controls',
        'Verify Tab wraps within the modal (focus trap)',
        'Press Escape — verify modal closes and focus returns',
        'Verify main page content is not Tab-accessible while modal is open',
      ],
      expected: 'Focus is trapped inside modal; Escape releases the trap',
      status: 'PENDING',
    },
    {
      id: 'KB-04',
      criterion: 'WCAG 2.4.1',
      description: 'Skip link functions correctly',
      steps: [
        'Load the page and immediately press Tab',
        'Verify skip link appears visually (top-centre of viewport)',
        'Press Enter — verify viewport scrolls to main content',
        'Verify focus lands on #main-content',
      ],
      expected: 'Skip link is visible on focus and moves focus to main content',
      status: 'PENDING',
    },
  ],

  'Colour & Visual': [
    {
      id: 'CV-01',
      criterion: 'WCAG 1.4.3',
      description: 'Verify text contrast ratios meet AA with browser DevTools',
      steps: [
        'Open Chrome DevTools → Elements → select body text elements',
        'Use the Accessibility panel to verify contrast ratios',
        'Check: primary text on card backgrounds (expect ≥4.5:1)',
        'Check: secondary text on card backgrounds (expect ≥4.5:1)',
        'Check: white text on accent/button backgrounds (expect ≥3.0:1 for large text)',
      ],
      expected: 'All text meets WCAG AA contrast thresholds',
      status: 'PENDING',
    },
    {
      id: 'CV-02',
      criterion: 'WCAG 1.4.11',
      description: 'UI component contrast meets 3:1',
      steps: [
        'Check focus ring against adjacent background colours',
        'Check form input borders against input background',
        'Check status indicator dots against card background',
        'Check toggle button border against page background',
      ],
      expected: 'All UI components have ≥3:1 contrast against adjacent colours',
      status: 'PENDING',
    },
    {
      id: 'CV-03',
      criterion: 'WCAG 1.4.4',
      description: 'Text resizes to 200% without loss of content',
      steps: [
        'Set browser text size to 200% (browser zoom or Ctrl/Cmd + zoom)',
        'Verify all text is still readable and not clipped',
        'Verify layout does not break at 200%',
        'Check nav, cards, tables, modals at 200%',
      ],
      expected: 'Page is readable and functional at 200% text size',
      status: 'PENDING',
    },
    {
      id: 'CV-04',
      criterion: 'WCAG 1.4.10',
      description: 'Page reflows correctly on narrow viewports (320px)',
      steps: [
        'Set browser width to 320px (DevTools responsive mode)',
        'Verify no horizontal scrollbar appears',
        'Verify all content is accessible without horizontal scrolling',
        'Check that tables, filters, and modals adapt to narrow width',
      ],
      expected: 'Content reflows with no horizontal scrolling at 320px',
      status: 'PENDING',
    },
  ],

  'Motion & Animation': [
    {
      id: 'MO-01',
      criterion: 'WCAG 2.3.3',
      description: 'Reduced motion preference disables animations',
      steps: [
        'Enable "Reduce Motion" in OS accessibility settings (or DevTools)',
        'Reload the page — verify no animations play on load',
        'Add events to the feed — verify no slide/fade animations',
        'Open/close modals — verify no transition animations',
        'Open toast notifications — verify instant appearance',
      ],
      expected: 'All animations are disabled when prefers-reduced-motion is active',
      status: 'PENDING',
    },
  ],

  'Forms & Error Handling': [
    {
      id: 'FE-01',
      criterion: 'WCAG 3.3.1',
      description: 'Form validation errors are identified and described',
      steps: [
        'Submit the dispute form empty — verify error messages appear',
        'Verify error messages are associated with their inputs',
        'Verify screen reader announces validation errors',
        'Verify invalid inputs are marked with aria-invalid="true"',
        'Correct errors and re-submit — verify success message',
      ],
      expected: 'Errors are programmatically identified and described',
      status: 'PENDING',
    },
    {
      id: 'FE-02',
      criterion: 'WCAG 3.3.2',
      description: 'Labels and instructions are provided before inputs',
      steps: [
        'Verify every form field has a visible label above/beside it',
        'Verify helper text (form-help spans) are linked via aria-describedby',
        'Verify required fields are indicated visually (asterisk) and programmatically (required attribute)',
      ],
      expected: 'All inputs have labels and appropriate instructions',
      status: 'PENDING',
    },
  ],
};

// ─── WCAG 2.1 AA Criterion Mapping ────────────────────────────────────────────

const WCAG_CRITERIA = [
  { id: '1.1.1', name: 'Non-text Content', level: 'A', automated: true },
  { id: '1.3.1', name: 'Info and Relationships', level: 'A', automated: true },
  { id: '1.3.2', name: 'Meaningful Sequence', level: 'A', automated: true },
  { id: '1.4.1', name: 'Use of Color', level: 'A', automated: true },
  { id: '1.4.3', name: 'Contrast (Minimum)', level: 'AA', automated: true },
  { id: '1.4.4', name: 'Resize Text', level: 'AA', automated: false, manual: 'CV-03' },
  { id: '1.4.10', name: 'Reflow', level: 'AA', automated: false, manual: 'CV-04' },
  { id: '1.4.11', name: 'Non-text Contrast', level: 'AA', automated: true },
  { id: '1.4.12', name: 'Text Spacing', level: 'AA', automated: false },
  { id: '1.4.13', name: 'Content on Hover or Focus', level: 'AA', automated: false },
  { id: '2.1.1', name: 'Keyboard', level: 'A', automated: true, manual: 'KB-01' },
  { id: '2.1.2', name: 'No Keyboard Trap', level: 'A', automated: true, manual: 'KB-03' },
  { id: '2.3.3', name: 'Animation from Interactions', level: 'AAA', automated: true, manual: 'MO-01' },
  { id: '2.4.1', name: 'Bypass Blocks', level: 'A', automated: true, manual: 'KB-04' },
  { id: '2.4.3', name: 'Focus Order', level: 'A', automated: true },
  { id: '2.4.4', name: 'Link Purpose (In Context)', level: 'A', automated: true },
  { id: '2.4.6', name: 'Headings and Labels', level: 'AA', automated: true },
  { id: '2.4.7', name: 'Focus Visible', level: 'AA', automated: true, manual: 'KB-02' },
  { id: '3.1.1', name: 'Language of Page', level: 'A', automated: true },
  { id: '3.1.2', name: 'Language of Parts', level: 'AA', automated: true, manual: 'SR-04' },
  { id: '3.2.1', name: 'On Focus', level: 'A', automated: true },
  { id: '3.2.2', name: 'On Input', level: 'A', automated: true },
  { id: '3.3.1', name: 'Error Identification', level: 'A', automated: false, manual: 'FE-01' },
  { id: '3.3.2', name: 'Labels or Instructions', level: 'A', automated: true, manual: 'FE-02' },
  { id: '4.1.1', name: 'Parsing', level: 'A', automated: true },
  { id: '4.1.2', name: 'Name, Role, Value', level: 'A', automated: true, manual: 'SR-01' },
  { id: '4.1.3', name: 'Status Messages', level: 'AA', automated: true },
];

// ─── Report Generator ─────────────────────────────────────────────────────────

function generateMarkdownReport(jestResults) {
  const now = new Date().toISOString();
  const jestSummary = jestResults
    ? `\n> **Automated Tests:** ${jestResults.numPassedTests} passed · ${jestResults.numFailedTests} failed · ${jestResults.numTotalTests} total\n`
    : '';

  const wcagTable = WCAG_CRITERIA.map((c) => {
    const tested = c.automated ? '✅ Automated' : c.manual ? `📋 Manual (${c.manual})` : '📋 Manual';
    const level = c.level === 'AA' ? '**AA**' : c.level;
    return `| ${c.id} | ${c.name} | ${level} | ${tested} |`;
  }).join('\n');

  const manualSections = Object.entries(MANUAL_CHECKLIST)
    .map(([category, items]) => {
      const rows = items
        .map(
          (item) => `
### ${item.id} — ${item.description}

**WCAG:** ${item.criterion}  
**Status:** ${item.status}

**Steps:**
${item.steps.map((s, i) => `${i + 1}. ${s}`).join('\n')}

**Expected result:** ${item.expected}
`
        )
        .join('\n');
      return `## ${category}\n${rows}`;
    })
    .join('\n---\n');

  return `# StellarEscrow Accessibility Report

**Generated:** ${now}  
**Standard:** WCAG 2.1 Level AA  
**Project:** @stellar-escrow/frontend
${jestSummary}
---

## WCAG 2.1 AA Criterion Coverage

| Criterion | Name | Level | Test Coverage |
|-----------|------|-------|---------------|
${wcagTable}

---

## Automated Test Suites

| Suite | File | Description |
|-------|------|-------------|
| ARIA & Landmarks | \`tests/aria.test.js\` | Landmark structure, ARIA attributes, roles, modal accessibility |
| Keyboard Navigation | \`tests/keyboard.test.js\` | Tab order, Escape/Arrow keys, mobile menu, form submission |
| Colour Contrast | \`tests/contrast.test.js\` | Contrast ratios, CSS tokens, visual helpers |
| Screen Reader | \`tests/screen-reader.test.js\` | Live regions, dynamic content, status updates |
| i18n & RTL | \`tests/i18n.test.js\` | Translation lookup, RTL DOM, locale persistence |

Run all automated tests:
\`\`\`bash
npm test
\`\`\`

Run with coverage:
\`\`\`bash
npm run test:coverage
\`\`\`

---

## Manual Testing Procedures

${manualSections}

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
- [ ] All images have \`alt\` attributes
- [ ] All form inputs have associated \`<label>\` or \`aria-label\`
- [ ] All interactive elements are reachable via Tab
- [ ] No positive \`tabindex\` values
- [ ] Skip link present and functional
- [ ] Correct heading hierarchy (no skipped levels)
- [ ] All \`<section>\` elements have accessible names
- [ ] All \`<nav>\` elements have \`aria-label\`
- [ ] Modals have \`role="dialog"\` + \`aria-modal="true"\` + \`aria-labelledby\`
- [ ] Tables have \`<caption>\` or \`aria-describedby\`
- [ ] Table headers use \`<th scope="col|row">\`

### ARIA
- [ ] Live regions present for dynamic content
- [ ] \`aria-expanded\` updated on menus/dropdowns
- [ ] \`aria-pressed\` used on toggle buttons
- [ ] \`aria-current="page"\` on active nav link
- [ ] \`aria-invalid\` set on errored inputs

### CSS / Visual
- [ ] \`:focus-visible\` styles present
- [ ] Focus ring has ≥3px outline
- [ ] High-contrast mode (\`[data-theme="high-contrast"]\`) functional
- [ ] \`prefers-reduced-motion\` media query disables all animations
- [ ] \`.visually-hidden\` class used for screen-reader-only text
- [ ] Colour is not the sole means of conveying information
- [ ] Print styles hide non-essential UI elements

### JavaScript
- [ ] \`announce()\` function used for all dynamic state changes
- [ ] Toast notifications have \`role="alert"\`
- [ ] Escape key closes open menus/modals
- [ ] Arrow keys navigate within composite widgets (tables, menus)
- [ ] Focus returns to trigger after modal closes
- [ ] Language change updates \`html[lang]\` and \`dir\`

---

## Known Limitations

1. **Automated contrast tests** use hex values from CSS tokens; browser-computed
   colours after theme inheritance are verified via manual DevTools audit.
2. **Focus trap** in modals is implemented in JavaScript; automated tests verify
   the correct DOM structure; manual KB-03 verifies runtime behaviour.
3. **Screen reader testing** requires human testers with assistive technology;
   automated tests verify the correct ARIA markup is present.

---

*Report generated by \`src/report.js\`. Update manual test statuses in the MANUAL_CHECKLIST object.*
`;
}

// ─── Main ─────────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
const jsonIndex = args.indexOf('--json');
let jestResults = null;

if (jsonIndex !== -1 && args[jsonIndex + 1]) {
  try {
    jestResults = JSON.parse(fs.readFileSync(args[jsonIndex + 1], 'utf8'));
  } catch (e) {
    console.error('Could not read Jest results JSON:', e.message);
  }
}

const report = generateMarkdownReport(jestResults);
const outPath = path.join(__dirname, '../ACCESSIBILITY_REPORT.md');
fs.writeFileSync(outPath, report, 'utf8');

console.log('✅ Accessibility report generated:', outPath);
console.log('');
console.log('Manual test categories:');
Object.keys(MANUAL_CHECKLIST).forEach((cat) => {
  const items = MANUAL_CHECKLIST[cat];
  console.log(`  ${cat}: ${items.length} procedure(s)`);
});
console.log('');
console.log('WCAG 2.1 AA criteria covered:', WCAG_CRITERIA.filter((c) => c.automated).length, '/ automated');
console.log('WCAG 2.1 AA criteria with manual tests:', WCAG_CRITERIA.filter((c) => c.manual).length);