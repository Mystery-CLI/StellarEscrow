import { test, expect } from '@playwright/test';

/**
 * Polyfill verification tests.
 * Ensures all polyfills injected by compat.js work correctly across browsers.
 */

test.describe('Polyfills', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('Element.closest works on a real DOM node', async ({ page }) => {
    const result = await page.evaluate(() => {
      const child = document.querySelector('main a') || document.querySelector('main button');
      if (!child) return 'no-element';
      const ancestor = child.closest('main');
      return ancestor ? ancestor.tagName.toLowerCase() : 'not-found';
    });
    expect(['main', 'no-element']).toContain(result);
  });

  test('Element.matches is a function', async ({ page }) => {
    const ok = await page.evaluate(() => typeof Element.prototype.matches === 'function');
    expect(ok).toBe(true);
  });

  test('NodeList.forEach is a function', async ({ page }) => {
    const ok = await page.evaluate(() => typeof NodeList.prototype.forEach === 'function');
    expect(ok).toBe(true);
  });

  test('Array.from is a function', async ({ page }) => {
    const ok = await page.evaluate(() => typeof Array.from === 'function');
    expect(ok).toBe(true);
  });

  test('Array.from converts NodeList', async ({ page }) => {
    const len = await page.evaluate(() => {
      const nodes = document.querySelectorAll('a');
      return Array.from(nodes).length;
    });
    expect(typeof len).toBe('number');
  });

  test('Object.assign is a function', async ({ page }) => {
    const ok = await page.evaluate(() => typeof Object.assign === 'function');
    expect(ok).toBe(true);
  });

  test('Object.assign merges objects correctly', async ({ page }) => {
    const result = await page.evaluate(() => {
      const a = { x: 1 };
      const b = { y: 2 };
      return Object.assign({}, a, b);
    });
    expect(result).toEqual({ x: 1, y: 2 });
  });

  test('CustomEvent can be constructed', async ({ page }) => {
    const ok = await page.evaluate(() => {
      try {
        const e = new CustomEvent('test', { detail: { v: 42 } });
        return e.type === 'test';
      } catch { return false; }
    });
    expect(ok).toBe(true);
  });

  test('requestAnimationFrame is a function', async ({ page }) => {
    const ok = await page.evaluate(() => typeof requestAnimationFrame === 'function');
    expect(ok).toBe(true);
  });
});
