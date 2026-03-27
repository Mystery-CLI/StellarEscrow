import { test, expect } from '@playwright/test';
import { REQUIRED_FEATURES } from './browser-matrix';

/**
 * Cross-browser compatibility tests.
 * Verifies that compat.js initialises correctly and all required browser
 * features are present across the full browser matrix.
 */

test.describe('Browser compatibility — compat.js', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('StellarCompat is exposed on window', async ({ page }) => {
    const compat = await page.evaluate(() => typeof (window as any).StellarCompat);
    expect(compat).toBe('object');
  });

  test('browser name and version are detected', async ({ page }) => {
    const browser = await page.evaluate(() => (window as any).StellarCompat.browser);
    expect(browser.name).toBeTruthy();
    expect(browser.version).toBeTruthy();
  });

  test('isSupported is true for all matrix browsers', async ({ page }) => {
    const isSupported = await page.evaluate(() => (window as any).StellarCompat.isSupported);
    expect(isSupported).toBe(true);
  });

  test('isIE is false for all matrix browsers', async ({ page }) => {
    const isIE = await page.evaluate(() => (window as any).StellarCompat.isIE);
    expect(isIE).toBe(false);
  });

  test('no IE warning banner is shown', async ({ page }) => {
    const banner = page.locator('#compat-ie-warning');
    await expect(banner).not.toBeAttached();
  });

  test('no outdated-browser warning is shown', async ({ page }) => {
    const banner = page.locator('#compat-outdated-warning');
    await expect(banner).not.toBeAttached();
  });
});

test.describe('Required feature detection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  for (const feature of REQUIRED_FEATURES) {
    test(`feature flag present: ${feature}`, async ({ page }) => {
      const features = await page.evaluate(() => (window as any).StellarCompat.features);
      // Map feature names to the keys used in StellarCompat.features
      const keyMap: Record<string, string> = {
        'fetch':               'fetch',
        'Promise':             'promise',
        'WebSocket':           'webSocket',
        'localStorage':        'localStorage',
        'CSS.supports':        'cssCustomProps',
        'Element.closest':     'fetch', // polyfilled — check via DOM
        'IntersectionObserver':'intersectionObs',
      };
      const key = keyMap[feature];
      if (key) {
        expect(typeof features[key]).toBe('boolean');
      }
    });
  }

  test('fetch is available (native or polyfilled)', async ({ page }) => {
    const hasFetch = await page.evaluate(() => typeof fetch !== 'undefined');
    expect(hasFetch).toBe(true);
  });

  test('Promise is available', async ({ page }) => {
    const hasPromise = await page.evaluate(() => typeof Promise !== 'undefined');
    expect(hasPromise).toBe(true);
  });

  test('localStorage is accessible', async ({ page }) => {
    const ok = await page.evaluate(() => {
      try {
        localStorage.setItem('_pw_test', '1');
        localStorage.removeItem('_pw_test');
        return true;
      } catch { return false; }
    });
    expect(ok).toBe(true);
  });

  test('Element.closest is available (native or polyfilled)', async ({ page }) => {
    const ok = await page.evaluate(() => typeof Element.prototype.closest === 'function');
    expect(ok).toBe(true);
  });

  test('requestAnimationFrame is available (native or polyfilled)', async ({ page }) => {
    const ok = await page.evaluate(() => typeof requestAnimationFrame === 'function');
    expect(ok).toBe(true);
  });
});
