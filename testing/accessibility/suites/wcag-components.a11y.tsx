/**
 * WCAG 2.1 AA Automated Accessibility Tests — Component Library
 *
 * Uses axe-core to scan rendered components for accessibility violations.
 * Each test maps to one or more WCAG success criteria.
 */

import React from 'react';
import { render } from '@testing-library/react';
import axe from 'axe-core';

// ---------------------------------------------------------------------------
// Helper: run axe on a rendered component
// ---------------------------------------------------------------------------

async function checkA11y(ui: React.ReactElement): Promise<axe.AxeResults> {
  const { container } = render(ui);
  return axe.run(container, {
    runOnly: {
      type: 'tag',
      values: ['wcag2a', 'wcag2aa', 'wcag21aa', 'best-practice'],
    },
  });
}

function expectNoViolations(results: axe.AxeResults) {
  const violations = results.violations;
  if (violations.length > 0) {
    const msg = violations.map(v =>
      `[${v.id}] ${v.description}\n  Impact: ${v.impact}\n  Nodes: ${v.nodes.map(n => n.html).join(', ')}`
    ).join('\n\n');
    throw new Error(`Accessibility violations found:\n\n${msg}`);
  }
}

// ---------------------------------------------------------------------------
// Button component
// ---------------------------------------------------------------------------

describe('A11y — Button (WCAG 4.1.2, 2.1.1)', () => {
  it('primary button has accessible name', async () => {
    const results = await checkA11y(<button type="button">Create Trade</button>);
    expectNoViolations(results);
  });

  it('icon-only button requires aria-label', async () => {
    const results = await checkA11y(
      <button type="button" aria-label="Close dialog">✕</button>
    );
    expectNoViolations(results);
  });

  it('disabled button is announced correctly', async () => {
    const results = await checkA11y(
      <button type="button" disabled aria-disabled="true">Submit</button>
    );
    expectNoViolations(results);
  });
});

// ---------------------------------------------------------------------------
// Form inputs
// ---------------------------------------------------------------------------

describe('A11y — Form Inputs (WCAG 1.3.1, 3.3.2)', () => {
  it('input with visible label passes', async () => {
    const results = await checkA11y(
      <div>
        <label htmlFor="seller">Seller Address</label>
        <input id="seller" type="text" />
      </div>
    );
    expectNoViolations(results);
  });

  it('required input has aria-required', async () => {
    const results = await checkA11y(
      <div>
        <label htmlFor="amount">Amount (USDC)</label>
        <input id="amount" type="number" required aria-required="true" />
      </div>
    );
    expectNoViolations(results);
  });

  it('input with error has aria-describedby', async () => {
    const results = await checkA11y(
      <div>
        <label htmlFor="buyer">Buyer Address</label>
        <input
          id="buyer"
          type="text"
          aria-invalid="true"
          aria-describedby="buyer-error"
        />
        <span id="buyer-error" role="alert">Must be a valid Stellar address</span>
      </div>
    );
    expectNoViolations(results);
  });
});

// ---------------------------------------------------------------------------
// Navigation landmark
// ---------------------------------------------------------------------------

describe('A11y — Navigation (WCAG 2.4.1, 1.3.1)', () => {
  it('nav has aria-label', async () => {
    const results = await checkA11y(
      <nav aria-label="Primary navigation">
        <ul>
          <li><a href="/dashboard">Dashboard</a></li>
          <li><a href="/trades">Trades</a></li>
        </ul>
      </nav>
    );
    expectNoViolations(results);
  });

  it('skip link is present and targets main', async () => {
    const results = await checkA11y(
      <div>
        <a href="#main-content" className="skip-link">Skip to main content</a>
        <nav aria-label="Primary"><a href="/">Home</a></nav>
        <main id="main-content"><p>Content</p></main>
      </div>
    );
    expectNoViolations(results);
  });
});

// ---------------------------------------------------------------------------
// Heading hierarchy
// ---------------------------------------------------------------------------

describe('A11y — Headings (WCAG 1.3.1, 2.4.6)', () => {
  it('page has single h1', async () => {
    const results = await checkA11y(
      <main>
        <h1>StellarEscrow Dashboard</h1>
        <section>
          <h2>Active Trades</h2>
          <h3>Trade #123</h3>
        </section>
      </main>
    );
    expectNoViolations(results);
  });
});

// ---------------------------------------------------------------------------
// Color contrast (structural — axe checks contrast ratios)
// ---------------------------------------------------------------------------

describe('A11y — Color Contrast (WCAG 1.4.3)', () => {
  it('text on dark background meets 4.5:1 ratio', async () => {
    const results = await checkA11y(
      <div style={{ backgroundColor: '#0f0f1a', color: '#e8e8f0', padding: '16px' }}>
        <p>Trade amount: 100 USDC</p>
      </div>
    );
    // axe checks contrast automatically
    const contrastViolations = results.violations.filter(v => v.id === 'color-contrast');
    expect(contrastViolations).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// ARIA live regions
// ---------------------------------------------------------------------------

describe('A11y — Live Regions (WCAG 4.1.3)', () => {
  it('status messages use aria-live polite', async () => {
    const results = await checkA11y(
      <div aria-live="polite" aria-atomic="true">
        <p>Trade created successfully</p>
      </div>
    );
    expectNoViolations(results);
  });

  it('error alerts use role="alert"', async () => {
    const results = await checkA11y(
      <div role="alert" aria-live="assertive">
        <p>Transaction failed. Please try again.</p>
      </div>
    );
    expectNoViolations(results);
  });
});

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

describe('A11y — Tables (WCAG 1.3.1)', () => {
  it('data table has headers and caption', async () => {
    const results = await checkA11y(
      <table>
        <caption>Recent Trades</caption>
        <thead>
          <tr>
            <th scope="col">Trade ID</th>
            <th scope="col">Amount</th>
            <th scope="col">Status</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>123</td>
            <td>100 USDC</td>
            <td>Completed</td>
          </tr>
        </tbody>
      </table>
    );
    expectNoViolations(results);
  });
});

// ---------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------

describe('A11y — Images (WCAG 1.1.1)', () => {
  it('informative image has alt text', async () => {
    const results = await checkA11y(
      <img src="/logo.png" alt="StellarEscrow logo" />
    );
    expectNoViolations(results);
  });

  it('decorative image has empty alt', async () => {
    const results = await checkA11y(
      <img src="/decoration.png" alt="" role="presentation" />
    );
    expectNoViolations(results);
  });
});
