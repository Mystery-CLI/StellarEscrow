/**
 * Browser Test Matrix — supported browsers and minimum versions.
 * Used by tests to assert compatibility expectations.
 */
export const BROWSER_MATRIX = {
  desktop: [
    { name: 'Chrome',  minVersion: 90,  engine: 'chromium' },
    { name: 'Firefox', minVersion: 88,  engine: 'firefox'  },
    { name: 'Safari',  minVersion: 14,  engine: 'webkit'   },
    { name: 'Edge',    minVersion: 90,  engine: 'chromium' },
  ],
  mobile: [
    { name: 'Chrome Android', device: 'Pixel 5',   engine: 'chromium' },
    { name: 'Safari iOS',     device: 'iPhone 12', engine: 'webkit'   },
  ],
} as const;

/** Features every supported browser must provide. */
export const REQUIRED_FEATURES = [
  'fetch',
  'Promise',
  'WebSocket',
  'localStorage',
  'CSS.supports',
  'Element.closest',
  'IntersectionObserver',
] as const;
