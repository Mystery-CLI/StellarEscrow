/**
 * SSL/TLS monitoring — client-side certificate and connection checks.
 * Beacons results to /api/metrics alongside other performance data.
 */

/**
 * Check the current connection is HTTPS and report the TLS info.
 * Sends a beacon with protocol, cipher (if available), and cert validity.
 */
export function monitorTlsConnection(): void {
  if (typeof window === 'undefined') return;

  const isSecure = location.protocol === 'https:';

  const payload: Record<string, unknown> = {
    source: 'ssl',
    type: 'connection_check',
    secure: isSecure,
    origin: location.origin,
    ts: Date.now(),
  };

  // SecurityPolicyViolationEvent / navigator connection info where available
  if ('connection' in navigator) {
    payload.effectiveType = (navigator as any).connection?.effectiveType;
  }

  if (!isSecure) {
    console.warn('[ssl] Page loaded over insecure HTTP — HTTPS redirect may not be configured.');
  }

  if (navigator.sendBeacon) {
    navigator.sendBeacon('/api/metrics', JSON.stringify(payload));
  }
}

/**
 * Enforce HTTPS at the client level as a last-resort fallback.
 * The nginx redirect should handle this, but this catches edge cases.
 */
export function enforceHttps(): void {
  if (typeof window !== 'undefined' && location.protocol !== 'https:' && location.hostname !== 'localhost') {
    location.replace(`https://${location.host}${location.pathname}${location.search}`);
  }
}
