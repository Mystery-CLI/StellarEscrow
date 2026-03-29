import { initializeSecurity } from './headers';
import { runComplianceChecks, runPenetrationTests, runSecurityAssessment, runVulnerabilityScan } from './assessment';
import { SecurityMonitor } from './monitoring';

describe('Security assessment', () => {
  beforeEach(() => {
    document.head.innerHTML = '';
    localStorage.clear();
    jest.restoreAllMocks();
  });

  it('executes penetration scenarios against protected browser flows', () => {
    initializeSecurity();

    const report = runPenetrationTests({
      htmlPayloads: [
        '<img src=x onerror="alert(1)">',
        '<script>alert(1)</script>',
        '<a href="javascript:alert(1)">Click me</a>',
      ],
      attributePayloads: ['javascript:alert(1)', 'onerror=alert(1)', 'onclick=alert(1)'],
      rateLimit: { maxAttempts: 2, attempts: 3, windowMs: 1000, key: 'attacker' },
      secureStorageCases: [{ key: 'session', value: { token: 'top-secret' }, encrypt: true }],
      csrf: { invalidToken: 'bad-token' },
    });

    expect(report.passed).toBe(true);
    expect(report.summary.failed).toBe(0);
    expect(report.summary.total).toBeGreaterThanOrEqual(6);
  });

  it('flags weak configurations during vulnerability scanning', () => {
    const monitor = new SecurityMonitor({ minimumSeverity: 'medium' });

    const report = runVulnerabilityScan(
      {
        csp: "default-src *; script-src 'self' 'unsafe-eval'; img-src *",
        endpoints: ['http://api.example.com'],
        headers: {
          'referrer-policy': 'unsafe-url',
        },
        encryptionKey: 'default-key-change-in-production',
      },
      monitor
    );

    expect(report.passed).toBe(false);
    expect(report.summary.failed).toBeGreaterThanOrEqual(3);
    expect(report.findings.find((finding) => finding.id === 'vulnerability-encryption-key')?.passed).toBe(false);
    expect(monitor.getAlerts().length).toBeGreaterThan(0);
  });

  it('validates compliance scenarios for hardened configurations', () => {
    const report = runComplianceChecks({
      csp:
        "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; connect-src 'self' https://api.stellar.org; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
      headers: {
        'x-content-type-options': 'nosniff',
        'x-frame-options': 'DENY',
        'x-xss-protection': '1; mode=block',
        'referrer-policy': 'strict-origin-when-cross-origin',
        'permissions-policy': 'geolocation=(), microphone=(), camera=()',
      },
      endpoints: ['https://api.stellar.org'],
      requireHttps: true,
      monitoring: {
        enabled: true,
        endpoint: '/api/security/alerts',
      },
      encryptionKey: 'production-key-that-is-long-enough',
    });

    expect(report.passed).toBe(true);
    expect(report.summary.failed).toBe(0);
  });

  it('records and flushes monitoring alerts for browser policy violations', () => {
    const sendBeacon = jest.fn().mockReturnValue(true);
    Object.defineProperty(navigator, 'sendBeacon', {
      configurable: true,
      value: sendBeacon,
    });

    const monitor = new SecurityMonitor({
      endpoint: '/api/security/alerts',
      minimumSeverity: 'medium',
    });

    const stopObserving = monitor.observePolicyViolations();
    const event = new Event('securitypolicyviolation');

    Object.defineProperty(event, 'violatedDirective', { configurable: true, value: 'script-src' });
    Object.defineProperty(event, 'blockedURI', { configurable: true, value: 'http://evil.test' });

    window.dispatchEvent(event);
    stopObserving();

    expect(monitor.getAlerts()).toHaveLength(1);
    expect(monitor.flush()).toBe(true);
    expect(sendBeacon).toHaveBeenCalledWith(
      '/api/security/alerts',
      expect.stringContaining('policy_violation')
    );
  });

  it('produces an end-to-end security assessment summary', () => {
    const result = runSecurityAssessment({
      csp:
        "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; connect-src 'self' https://api.stellar.org; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
      headers: {
        'x-content-type-options': 'nosniff',
        'x-frame-options': 'DENY',
        'x-xss-protection': '1; mode=block',
        'referrer-policy': 'strict-origin-when-cross-origin',
        'permissions-policy': 'geolocation=(), microphone=(), camera=()',
      },
      endpoints: ['https://api.stellar.org'],
      requireHttps: true,
      encryptionKey: 'production-key-that-is-long-enough',
      monitoring: {
        enabled: true,
        endpoint: '/api/security/alerts',
      },
      rateLimit: { maxAttempts: 2, attempts: 3, windowMs: 1000, key: 'client' },
      secureStorageCases: [{ key: 'token', value: { value: 'secret' }, encrypt: true }],
    });

    expect(result.overallPassed).toBe(true);
    expect(result.reports).toHaveLength(3);
    expect(result.totals.failed).toBe(0);
    expect(result.monitoring.totalEvents).toBe(result.findings.length);
  });
});
