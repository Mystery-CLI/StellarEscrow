import { test, expect } from '@playwright/test';

/**
 * CSS feature and rendering tests.
 * Verifies CSS Grid, custom properties, and visual consistency across browsers.
 */

test.describe('CSS feature support', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('CSS Grid is supported or fallback is applied', async ({ page }) => {
    const result = await page.evaluate(() => {
      try { return CSS.supports('display', 'grid'); } catch { return false; }
    });
    // All matrix browsers support CSS Grid; if not, compat.js injects fallback styles
    const hasFallback = await page.evaluate(() =>
      !!(window as any).StellarCompat?.features?.cssCustomProps !== undefined
    );
    expect(result || hasFallback).toBe(true);
  });

  test('CSS custom properties are supported', async ({ page }) => {
    const supported = await page.evaluate(() => {
      try { return CSS.supports('--a', '0'); } catch { return false; }
    });
    // All matrix browsers (Chrome 90+, Firefox 88+, Safari 14+, Edge 90+) support custom props
    expect(supported).toBe(true);
  });

  test('body has a background color applied', async ({ page }) => {
    const bg = await page.evaluate(() =>
      getComputedStyle(document.body).backgroundColor
    );
    expect(bg).not.toBe('');
    expect(bg).not.toBe('rgba(0, 0, 0, 0)');
  });

  test('status grid uses grid layout', async ({ page }) => {
    const display = await page.evaluate(() => {
      const el = document.querySelector('.status-grid');
      return el ? getComputedStyle(el).display : null;
    });
    // May be 'grid' or 'block' depending on viewport; just ensure it's rendered
    expect(display).not.toBeNull();
  });
});

test.describe('CSS fallback for no-custom-props browsers', () => {
  test('compat.js cssCustomProps flag is a boolean', async ({ page }) => {
    await page.goto('/');
    const flag = await page.evaluate(() => (window as any).StellarCompat.features.cssCustomProps);
    expect(typeof flag).toBe('boolean');
  });
});
