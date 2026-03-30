/**
 * env.ts — Validate required environment variables at application startup.
 *
 * Vite replaces `import.meta.env.*` at build time. Any missing required
 * variable will cause this module to throw, preventing the app from mounting
 * with a broken configuration.
 *
 * Required variables (set in .env / .env.production / etc.):
 *   VITE_INDEXER_URL   — Base URL of the indexer API (e.g. https://api.stellarescrow.app)
 *
 * Optional variables:
 *   VITE_API_MOCK      — Set to "true" to use mock API (no backend needed)
 *   VITE_VAPID_PUBLIC_KEY — VAPID public key for push notifications
 */

interface EnvConfig {
  indexerUrl: string;
  apiMock: boolean;
  vapidPublicKey: string | undefined;
  network: 'testnet' | 'mainnet';
}

function validateEnv(): EnvConfig {
  const errors: string[] = [];

  const indexerUrl = import.meta.env.VITE_INDEXER_URL as string | undefined;
  if (!indexerUrl || indexerUrl.trim() === '') {
    errors.push(
      'VITE_INDEXER_URL is required — set it to the indexer API base URL (e.g. http://localhost:3000)'
    );
  }

  const networkRaw = (import.meta.env.VITE_STELLAR_NETWORK as string | undefined) ?? 'testnet';
  if (networkRaw !== 'testnet' && networkRaw !== 'mainnet') {
    errors.push(
      `VITE_STELLAR_NETWORK must be 'testnet' or 'mainnet', got '${networkRaw}'`
    );
  }

  if (errors.length > 0) {
    const msg = [
      '[StellarEscrow] Missing or invalid environment variables:',
      ...errors.map((e) => `  • ${e}`),
      '',
      'Copy .env.example to .env and fill in the required values.',
    ].join('\n');

    // Log to console so it's visible in the browser devtools
    console.error(msg);

    // In production builds, throw so the app fails visibly rather than silently
    if (import.meta.env.PROD) {
      throw new Error(msg);
    }
  }

  return {
    indexerUrl: indexerUrl ?? 'http://localhost:3000',
    apiMock: import.meta.env.VITE_API_MOCK === 'true',
    vapidPublicKey: import.meta.env.VITE_VAPID_PUBLIC_KEY as string | undefined,
    network: (networkRaw as 'testnet' | 'mainnet') ?? 'testnet',
  };
}

/** Validated, typed environment configuration. Throws on startup if required vars are missing. */
export const env = validateEnv();
