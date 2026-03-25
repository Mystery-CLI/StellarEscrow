/**
 * Browser Compatibility Tests
 * Tests for compat.js: detection, polyfills, feature flags, and warnings.
 */

// Minimal DOM/browser environment stubs
beforeEach(() => {
  document.body.innerHTML = '';
  delete window.StellarCompat;
  // Reset polyfill targets
  delete Element.prototype._origClosest;
});

// Helper: load compat.js in the current jsdom environment
function loadCompat() {
  jest.resetModules();
  require('../compat.js');
}

describe('Browser detection', () => {
  const originalUA = navigator.userAgent;

  afterEach(() => {
    Object.defineProperty(navigator, 'userAgent', { value: originalUA, configurable: true });
  });

  test('detects Chrome', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/110.0.0.0 Safari/537.36',
      configurable: true,
    });
    loadCompat();
    expect(window.StellarCompat.browser.name).toBe('Chrome');
    expect(parseInt(window.StellarCompat.browser.version)).toBeGreaterThanOrEqual(90);
  });

  test('detects Firefox', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0',
      configurable: true,
    });
    loadCompat();
    expect(window.StellarCompat.browser.name).toBe('Firefox');
  });

  test('detects IE and marks as unsupported', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.1; Trident/6.0)',
      configurable: true,
    });
    loadCompat();
    expect(window.StellarCompat.isIE).toBe(true);
    expect(window.StellarCompat.isSupported).toBe(false);
  });

  test('marks modern Chrome as supported', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 Chrome/110.0.0.0',
      configurable: true,
    });
    loadCompat();
    expect(window.StellarCompat.isSupported).toBe(true);
  });
});

describe('Feature detection', () => {
  test('exposes feature flags object', () => {
    loadCompat();
    const f = window.StellarCompat.features;
    expect(typeof f.webSocket).toBe('boolean');
    expect(typeof f.fetch).toBe('boolean');
    expect(typeof f.promise).toBe('boolean');
    expect(typeof f.localStorage).toBe('boolean');
    expect(typeof f.cssGrid).toBe('boolean');
    expect(typeof f.cssCustomProps).toBe('boolean');
  });

  test('detects WebSocket support', () => {
    loadCompat();
    // jsdom provides WebSocket stub
    expect(window.StellarCompat.features.webSocket).toBe(typeof WebSocket !== 'undefined');
  });
});

describe('Polyfills', () => {
  test('Element.closest is defined after load', () => {
    loadCompat();
    expect(typeof Element.prototype.closest).toBe('function');
  });

  test('Element.matches is defined after load', () => {
    loadCompat();
    expect(typeof Element.prototype.matches).toBe('function');
  });

  test('NodeList.forEach is defined after load', () => {
    loadCompat();
    expect(typeof NodeList.prototype.forEach).toBe('function');
  });

  test('Array.from is defined after load', () => {
    loadCompat();
    expect(typeof Array.from).toBe('function');
  });

  test('CustomEvent constructor works', () => {
    loadCompat();
    const evt = new CustomEvent('test', { detail: { x: 1 } });
    expect(evt.type).toBe('test');
  });

  test('requestAnimationFrame is defined', () => {
    loadCompat();
    expect(typeof window.requestAnimationFrame).toBe('function');
  });
});

describe('Compatibility warnings', () => {
  test('shows IE warning for IE user agent', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (compatible; MSIE 11.0; Windows NT 6.1; Trident/7.0)',
      configurable: true,
    });
    loadCompat();
    expect(document.getElementById('compat-ie-warning')).not.toBeNull();
  });

  test('does not show IE warning for Chrome', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 Chrome/110.0.0.0',
      configurable: true,
    });
    loadCompat();
    expect(document.getElementById('compat-ie-warning')).toBeNull();
  });

  test('warning banner has role=alert', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (compatible; MSIE 11.0; Trident/7.0)',
      configurable: true,
    });
    loadCompat();
    const banner = document.getElementById('compat-ie-warning');
    expect(banner).not.toBeNull();
    expect(banner.getAttribute('role')).toBe('alert');
  });

  test('warning banner has a dismiss button', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (compatible; MSIE 11.0; Trident/7.0)',
      configurable: true,
    });
    loadCompat();
    const banner = document.getElementById('compat-ie-warning');
    const btn = banner && banner.querySelector('button');
    expect(btn).not.toBeNull();
  });

  test('dismiss button removes the banner', () => {
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (compatible; MSIE 11.0; Trident/7.0)',
      configurable: true,
    });
    loadCompat();
    const banner = document.getElementById('compat-ie-warning');
    const btn = banner.querySelector('button');
    btn.click();
    expect(document.getElementById('compat-ie-warning')).toBeNull();
  });
});
