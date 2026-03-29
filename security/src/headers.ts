export const setupCSP = (): void => {
  const meta = document.createElement('meta');
  meta.httpEquiv = 'Content-Security-Policy';
  meta.content = `
    default-src 'self';
    script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net;
    style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
    font-src 'self' https://fonts.gstatic.com;
    img-src 'self' data: https:;
    connect-src 'self' https://api.stellar.org https://horizon.stellar.org;
    frame-ancestors 'none';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    upgrade-insecure-requests;
  `.replace(/\s+/g, ' ');
  document.head.appendChild(meta);
};

export const setupSecurityHeaders = (): void => {
  const xContentType = document.createElement('meta');
  xContentType.httpEquiv = 'X-Content-Type-Options';
  xContentType.content = 'nosniff';
  document.head.appendChild(xContentType);

  const xFrameOptions = document.createElement('meta');
  xFrameOptions.httpEquiv = 'X-Frame-Options';
  xFrameOptions.content = 'DENY';
  document.head.appendChild(xFrameOptions);

  const referrer = document.createElement('meta');
  referrer.httpEquiv = 'Referrer-Policy';
  referrer.content = 'strict-origin-when-cross-origin';
  document.head.appendChild(referrer);

  const permissions = document.createElement('meta');
  permissions.httpEquiv = 'Permissions-Policy';
  permissions.content = 'geolocation=(), microphone=(), camera=()';
  document.head.appendChild(permissions);

  const crossOriginOpener = document.createElement('meta');
  crossOriginOpener.httpEquiv = 'Cross-Origin-Opener-Policy';
  crossOriginOpener.content = 'same-origin';
  document.head.appendChild(crossOriginOpener);
};

export const setupXSSProtection = (): void => {
  // Disable inline scripts
  if (typeof window !== 'undefined') {
    (window as any).__CSP_NONCE__ = Math.random().toString(36).substring(2, 15);
  }

  // Set X-XSS-Protection header via meta
  const xssProtection = document.createElement('meta');
  xssProtection.httpEquiv = 'X-XSS-Protection';
  xssProtection.content = '1; mode=block';
  document.head.appendChild(xssProtection);
};

export const initializeSecurity = (): void => {
  setupCSP();
  setupSecurityHeaders();
  setupXSSProtection();
};
