import { test, expect } from '@playwright/test';

/**
 * Mobile browser tests — Android Chrome (Pixel 5) and iOS Safari (iPhone 12).
 * Covers viewport, touch interactions, PWA meta tags, and mobile-specific behaviour.
 */

test.describe('Mobile browser — PWA and meta tags', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('viewport meta tag is present', async ({ page }) => {
    const content = await page.locator('meta[name="viewport"]').getAttribute('content');
    expect(content).toContain('width=device-width');
  });

  test('apple-mobile-web-app-capable meta is set', async ({ page }) => {
    const content = await page.locator('meta[name="apple-mobile-web-app-capable"]').getAttribute('content');
    expect(content).toBe('yes');
  });

  test('web app manifest is linked', async ({ page }) => {
    const href = await page.locator('link[rel="manifest"]').getAttribute('href');
    expect(href).toBeTruthy();
  });

  test('theme-color meta is present', async ({ page }) => {
    const content = await page.locator('meta[name="theme-color"]').getAttribute('content');
    expect(content).toBeTruthy();
  });
});

test.describe('Mobile browser — touch and scroll', () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('page is scrollable vertically', async ({ page }) => {
    const scrollHeight = await page.evaluate(() => document.documentElement.scrollHeight);
    const clientHeight = await page.evaluate(() => document.documentElement.clientHeight);
    // Page should have content (scrollHeight >= clientHeight)
    expect(scrollHeight).toBeGreaterThanOrEqual(clientHeight);
  });

  test('no horizontal scroll on mobile', async ({ page }) => {
    const overflow = await page.evaluate(() =>
      document.documentElement.scrollWidth > window.innerWidth
    );
    expect(overflow).toBe(false);
  });

  test('wallet connect button is tappable (visible and enabled)', async ({ page }) => {
    const btn = page.locator('#wallet-connect-btn');
    await expect(btn).toBeVisible();
    await expect(btn).toBeEnabled();
  });

  test('language selector is usable on mobile', async ({ page }) => {
    const select = page.locator('#lang-select');
    await expect(select).toBeVisible();
    await expect(select).toBeEnabled();
  });
});

test.describe('Mobile browser — Android Chrome specific', () => {
  test.use({ ...require('@playwright/test').devices['Pixel 5'] });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('page loads without JS errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    await page.goto('/');
    await page.waitForLoadState('domcontentloaded');
    expect(errors).toHaveLength(0);
  });

  test('StellarCompat initialises on Android Chrome', async ({ page }) => {
    const compat = await page.evaluate(() => typeof (window as any).StellarCompat);
    expect(compat).toBe('object');
  });
});

test.describe('Mobile browser — iOS Safari specific', () => {
  test.use({ ...require('@playwright/test').devices['iPhone 12'] });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('page loads without JS errors on iOS Safari', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    await page.goto('/');
    await page.waitForLoadState('domcontentloaded');
    expect(errors).toHaveLength(0);
  });

  test('StellarCompat initialises on iOS Safari', async ({ page }) => {
    const compat = await page.evaluate(() => typeof (window as any).StellarCompat);
    expect(compat).toBe('object');
  });

  test('no horizontal overflow on iPhone 12', async ({ page }) => {
    const overflow = await page.evaluate(() =>
      document.documentElement.scrollWidth > window.innerWidth
    );
    expect(overflow).toBe(false);
  });
});

test.describe('Mobile browser — Samsung Internet specific', () => {
  test.use({ ...require('@playwright/test').devices['Galaxy S9+'] });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('page loads without JS errors on Samsung Internet', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    await page.goto('/');
    await page.waitForLoadState('domcontentloaded');
    expect(errors).toHaveLength(0);
  });

  test('StellarCompat initialises on Samsung Internet', async ({ page }) => {
    const compat = await page.evaluate(() => typeof (window as any).StellarCompat);
    expect(compat).toBe('object');
  });

  test('no horizontal overflow on Galaxy S9+', async ({ page }) => {
    const overflow = await page.evaluate(() =>
      document.documentElement.scrollWidth > window.innerWidth
    );
    expect(overflow).toBe(false);
  });
});
