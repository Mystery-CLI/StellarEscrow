import { test, expect } from '@playwright/test';

/**
 * Cross-browser navigation and layout tests.
 * Ensures core UI elements render and are interactive across all browsers.
 */

test.describe('Page load and core layout', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('page title is set', async ({ page }) => {
    await expect(page).toHaveTitle(/StellarEscrow/i);
  });

  test('main landmark is present', async ({ page }) => {
    await expect(page.locator('main#main-content')).toBeVisible();
  });

  test('header navigation renders', async ({ page }) => {
    await expect(page.locator('header nav')).toBeVisible();
  });

  test('skip-to-content link is in the DOM', async ({ page }) => {
    const skip = page.locator('a.skip-link');
    await expect(skip).toBeAttached();
  });

  test('dashboard section is visible', async ({ page }) => {
    await expect(page.locator('#dashboard')).toBeVisible();
  });

  test('wallet connect button is present', async ({ page }) => {
    await expect(page.locator('#wallet-connect-btn')).toBeVisible();
  });
});

test.describe('Mobile layout', () => {
  test.use({ viewport: { width: 390, height: 844 } }); // iPhone 12 dimensions

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('mobile menu button is visible on small screens', async ({ page }) => {
    // The hamburger button should be present in the DOM on mobile
    const menuBtn = page.locator('#mobile-menu-btn');
    await expect(menuBtn).toBeAttached();
  });

  test('page does not overflow horizontally', async ({ page }) => {
    const overflow = await page.evaluate(() => {
      return document.documentElement.scrollWidth > document.documentElement.clientWidth;
    });
    expect(overflow).toBe(false);
  });

  test('touch targets are at least 44px', async ({ page }) => {
    // Check the wallet connect button meets minimum touch target size
    const btn = page.locator('#wallet-connect-btn');
    const box = await btn.boundingBox();
    if (box) {
      expect(box.height).toBeGreaterThanOrEqual(44);
    }
  });
});

test.describe('Navigation links', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('dashboard nav link is present', async ({ page }) => {
    await expect(page.locator('a[href="#dashboard"]')).toBeAttached();
  });

  test('language selector is present', async ({ page }) => {
    await expect(page.locator('#lang-select')).toBeVisible();
  });

  test('language selector has expected options', async ({ page }) => {
    const options = await page.locator('#lang-select option').allTextContents();
    expect(options).toContain('English');
    expect(options).toContain('Français');
    expect(options).toContain('Español');
  });
});
