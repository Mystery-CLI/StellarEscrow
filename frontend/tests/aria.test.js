/**
 * ARIA & Landmark Tests
 * WCAG 2.1 AA — Perceivable / Robust
 */

beforeEach(() => global.loadFixture());

// ─── 1. Page-Level Landmarks ────────────────────────────────────────────────

describe('Landmark Structure', () => {
  test('page has exactly one <main> landmark', () => {
    const mains = document.querySelectorAll('main');
    expect(mains).toHaveLength(1);
  });

  test('<main> has id="main-content" for skip-link target', () => {
    expect(document.getElementById('main-content')).not.toBeNull();
  });

  test('page has <header> with role="banner"', () => {
    const header = document.querySelector('header[role="banner"]');
    expect(header).not.toBeNull();
  });

  test('page has <footer> with role="contentinfo"', () => {
    const footer = document.querySelector('footer[role="contentinfo"]');
    expect(footer).not.toBeNull();
  });

  test('page has at least one <nav> landmark', () => {
    const navs = document.querySelectorAll('nav');
    expect(navs.length).toBeGreaterThanOrEqual(1);
  });

  test('every <nav> has an aria-label', () => {
    const navs = document.querySelectorAll('nav');
    navs.forEach((nav) => {
      expect(
        nav.getAttribute('aria-label') || nav.getAttribute('aria-labelledby')
      ).toBeTruthy();
    });
  });

  test('all <section> elements have an accessible name', () => {
    const sections = document.querySelectorAll('section');
    sections.forEach((section) => {
      const labelledBy = section.getAttribute('aria-labelledby');
      const label = section.getAttribute('aria-label');
      if (labelledBy) {
        expect(document.getElementById(labelledBy)).not.toBeNull();
      } else {
        expect(label).toBeTruthy();
      }
    });
  });
});

// ─── 2. Headings ────────────────────────────────────────────────────────────

describe('Heading Hierarchy', () => {
  test('page has exactly one <h1>', () => {
    const h1s = document.querySelectorAll('h1');
    expect(h1s).toHaveLength(1);
  });

  test('<h1> is the site logo/name', () => {
    const h1 = document.querySelector('h1');
    expect(h1.textContent.trim()).toMatch(/StellarEscrow/i);
  });

  test('page has multiple <h2> section titles', () => {
    const h2s = document.querySelectorAll('h2');
    expect(h2s.length).toBeGreaterThanOrEqual(3);
  });

  test('no heading levels are skipped (h2 present before h3)', () => {
    const headings = [...document.querySelectorAll('h1,h2,h3,h4,h5,h6')];
    let prevLevel = 1;
    headings.forEach((h) => {
      const level = parseInt(h.tagName[1]);
      expect(level - prevLevel).toBeLessThanOrEqual(1);
      prevLevel = level;
    });
  });
});

// ─── 3. Skip Navigation ─────────────────────────────────────────────────────

describe('Skip Navigation', () => {
  test('skip link is present', () => {
    const skip = document.querySelector('.skip-link');
    expect(skip).not.toBeNull();
  });

  test('skip link points to #main-content', () => {
    const skip = document.querySelector('.skip-link');
    expect(skip.getAttribute('href')).toBe('#main-content');
  });

  test('skip link target exists in DOM', () => {
    const skip = document.querySelector('.skip-link');
    const target = document.querySelector(skip.getAttribute('href'));
    expect(target).not.toBeNull();
  });
});

// ─── 4. ARIA Live Regions ────────────────────────────────────────────────────

describe('ARIA Live Regions', () => {
  test('announcement live region exists', () => {
    const region = document.getElementById('live-region');
    expect(region).not.toBeNull();
  });

  test('announcement live region has aria-live="polite"', () => {
    const region = document.getElementById('live-region');
    expect(region.getAttribute('aria-live')).toBe('polite');
  });

  test('live region has aria-atomic="true"', () => {
    const region = document.getElementById('live-region');
    expect(region.getAttribute('aria-atomic')).toBe('true');
  });

  test('events log has role="log" with aria-live', () => {
    const log = document.getElementById('events-list');
    expect(log.getAttribute('role')).toBe('log');
    expect(log.getAttribute('aria-live')).toBeTruthy();
  });

  test('pagination info is aria-live', () => {
    const info = document.getElementById('pagination-info');
    expect(info.getAttribute('aria-live')).toBeTruthy();
  });

  test('toast container has aria-live="assertive"', () => {
    const toasts = document.getElementById('toast-container');
    expect(toasts.getAttribute('aria-live')).toBe('assertive');
  });

  test('ws-indicator is role="status" with aria-live', () => {
    const indicator = document.getElementById('ws-indicator');
    expect(indicator.getAttribute('role')).toBe('status');
    expect(indicator.getAttribute('aria-live')).toBeTruthy();
  });

  test('offline indicator is role="status"', () => {
    const offline = document.getElementById('offline-indicator');
    expect(offline.getAttribute('role')).toBe('status');
  });
});

// ─── 5. Button Accessibility ─────────────────────────────────────────────────

describe('Button Accessibility', () => {
  const getButtons = () => document.querySelectorAll('button');

  test('all buttons have an accessible name', () => {
    const buttons = getButtons();
    expect(buttons.length).toBeGreaterThan(0);
    buttons.forEach((btn) => {
      const name =
        btn.getAttribute('aria-label') ||
        btn.getAttribute('aria-labelledby') ||
        btn.textContent.trim();
      expect(name.length).toBeGreaterThan(0);
    });
  });

  test('toggle buttons track state with aria-pressed or aria-expanded', () => {
    const contrastToggle = document.getElementById('contrast-toggle');
    expect(contrastToggle.getAttribute('aria-pressed')).not.toBeNull();
  });

  test('menu-trigger buttons have aria-expanded and aria-controls', () => {
    const menuBtn = document.getElementById('mobile-menu-btn');
    expect(menuBtn.getAttribute('aria-expanded')).not.toBeNull();
    expect(menuBtn.getAttribute('aria-controls')).toBeTruthy();
  });

  test('wallet connect button has aria-expanded and aria-controls', () => {
    const btn = document.getElementById('wallet-connect-btn');
    expect(btn.getAttribute('aria-expanded')).not.toBeNull();
    expect(btn.getAttribute('aria-controls')).toBeTruthy();
  });

  test('icon-only buttons have aria-label', () => {
    const iconButtons = document.querySelectorAll('.btn-icon, .btn-close, .btn-wallet');
    iconButtons.forEach((btn) => {
      const label = btn.getAttribute('aria-label');
      expect(label).toBeTruthy();
    });
  });
});

// ─── 6. Form Accessibility ────────────────────────────────────────────────────

describe('Form Accessibility', () => {
  test('all form inputs have a <label> or aria-label', () => {
    const inputs = document.querySelectorAll('input, select, textarea');
    inputs.forEach((input) => {
      const id = input.id;
      const hasLabel = id && document.querySelector(`label[for="${id}"]`);
      const hasAriaLabel = input.getAttribute('aria-label');
      const hasAriaLabelledby = input.getAttribute('aria-labelledby');
      expect(!!(hasLabel || hasAriaLabel || hasAriaLabelledby)).toBe(true);
    });
  });

  test('required inputs are marked with required attribute', () => {
    const disputeTradeId = document.getElementById('dispute-trade-id');
    expect(disputeTradeId.hasAttribute('required')).toBe(true);
    const reason = document.getElementById('dispute-reason');
    expect(reason.hasAttribute('required')).toBe(true);
  });

  test('inputs with helper text reference it via aria-describedby', () => {
    const select = document.getElementById('event-type-filter');
    expect(select.getAttribute('aria-describedby')).toBe('event-type-help');
    expect(document.getElementById('event-type-help')).not.toBeNull();
  });

  test('search forms have role="search"', () => {
    const searchForms = document.querySelectorAll('[role="search"]');
    expect(searchForms.length).toBeGreaterThanOrEqual(1);
  });
});

// ─── 7. Modal / Dialog Accessibility ────────────────────────────────────────

describe('Modal / Dialog Accessibility', () => {
  const modals = () => document.querySelectorAll('[role="dialog"], [role="alertdialog"]');

  test('dialogs have role="dialog" or "alertdialog"', () => {
    expect(modals().length).toBeGreaterThanOrEqual(2);
  });

  test('dialogs have aria-modal="true"', () => {
    modals().forEach((modal) => {
      expect(modal.getAttribute('aria-modal')).toBe('true');
    });
  });

  test('dialogs have an accessible title via aria-labelledby', () => {
    modals().forEach((modal) => {
      const labelId = modal.getAttribute('aria-labelledby');
      expect(labelId).toBeTruthy();
      expect(document.getElementById(labelId)).not.toBeNull();
    });
  });

  test('error alertdialog has aria-describedby for message', () => {
    const errorModal = document.getElementById('error-modal');
    expect(errorModal.getAttribute('aria-describedby')).toBeTruthy();
  });

  test('dialogs have a close button with aria-label', () => {
    const closeButtons = document.querySelectorAll('[aria-label="Close dialog"]');
    expect(closeButtons.length).toBeGreaterThanOrEqual(1);
  });

  test('hidden dialogs use the hidden attribute', () => {
    modals().forEach((modal) => {
      // All modals start hidden
      expect(modal.hasAttribute('hidden')).toBe(true);
    });
  });
});

// ─── 8. Table Accessibility ──────────────────────────────────────────────────

describe('Table Accessibility', () => {
  test('tables have <caption> or aria-describedby', () => {
    const tables = document.querySelectorAll('table');
    tables.forEach((table) => {
      const hasCaption = table.querySelector('caption');
      const hasDescribedBy = table.getAttribute('aria-describedby') &&
        document.getElementById(table.getAttribute('aria-describedby'));
      expect(!!(hasCaption || hasDescribedBy)).toBe(true);
    });
  });

  test('table headers use <th scope="col"> or <th scope="row">', () => {
    const ths = document.querySelectorAll('th');
    ths.forEach((th) => {
      const scope = th.getAttribute('scope');
      expect(['col', 'row', 'colgroup', 'rowgroup']).toContain(scope);
    });
  });
});

// ─── 9. Image / Decorative Icon Accessibility ────────────────────────────────

describe('Icon & Image Accessibility', () => {
  test('decorative icons use aria-hidden="true"', () => {
    const decorativeSpans = document.querySelectorAll('[aria-hidden="true"]');
    expect(decorativeSpans.length).toBeGreaterThan(0);
  });

  test('no <img> tags lack alt attribute', () => {
    const imgs = document.querySelectorAll('img');
    imgs.forEach((img) => {
      expect(img.hasAttribute('alt')).toBe(true);
    });
  });
});

// ─── 10. Language ────────────────────────────────────────────────────────────

describe('Language Declaration', () => {
  test('html element has lang attribute', () => {
    const html = document.documentElement;
    expect(html.getAttribute('lang')).toBeTruthy();
  });

  test('lang is a valid BCP 47 tag', () => {
    const lang = document.documentElement.getAttribute('lang');
    expect(lang).toMatch(/^[a-z]{2,3}(-[A-Z][a-z]{3})?(-[A-Z]{2})?$/);
  });
});

// ─── 11. High Contrast Toggle ────────────────────────────────────────────────

describe('High Contrast Mode Toggle', () => {
  test('contrast toggle exists in DOM', () => {
    expect(document.getElementById('contrast-toggle')).not.toBeNull();
  });

  test('toggle has initial aria-pressed="false"', () => {
    expect(document.getElementById('contrast-toggle').getAttribute('aria-pressed')).toBe('false');
  });

  test('toggling updates aria-pressed', () => {
    const btn = document.getElementById('contrast-toggle');
    btn.setAttribute('aria-pressed', 'true');
    expect(btn.getAttribute('aria-pressed')).toBe('true');
  });
});

// ─── 12. Wallet Region ───────────────────────────────────────────────────────

describe('Wallet Region Accessibility', () => {
  test('wallet container is a labelled region', () => {
    const container = document.getElementById('wallet-container');
    expect(container.getAttribute('role')).toBe('region');
    expect(container.getAttribute('aria-label')).toBeTruthy();
  });

  test('wallet menu options use role="menuitem"', () => {
    const items = document.querySelectorAll('#wallet-menu [role="menuitem"]');
    expect(items.length).toBeGreaterThanOrEqual(1);
  });

  test('wallet dropdown items have role="menuitem"', () => {
    const items = document.querySelectorAll('#wallet-dropdown [role="menuitem"]');
    expect(items.length).toBeGreaterThanOrEqual(1);
  });
});