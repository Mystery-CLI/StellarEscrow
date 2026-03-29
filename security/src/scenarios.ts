import { secureStorage } from './storage';
import { sanitizeHtml } from './sanitization';
import { RateLimiter } from './validation';
import { CSRFProtection, XSSProtection } from './xss';
import { SecurityMonitor } from './monitoring';
import { SecurityFinding, SecurityScenario, SecurityTestTarget } from './types';

const DEFAULT_HTML_PAYLOADS = [
  '<img src=x onerror="alert(1)">',
  '<script>alert(1)</script>',
  '<a href="javascript:alert(1)">Click me</a>',
];

const DEFAULT_ATTRIBUTE_PAYLOADS = ['javascript:alert(1)', 'onerror=alert(1)', 'onclick = alert(1)'];

const DEFAULT_SECURITY_HEADERS: Record<string, string> = {
  'x-content-type-options': 'nosniff',
  'x-frame-options': 'DENY',
  'x-xss-protection': '1; mode=block',
  'referrer-policy': 'strict-origin-when-cross-origin',
  'permissions-policy': 'geolocation=(), microphone=(), camera=()',
};

const DEFAULT_CSP =
  "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' https://api.stellar.org; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";

const DANGEROUS_PATTERN = /<script|javascript:|on\w+\s*=|<iframe|srcdoc=|data:text\/html/i;
const DEFAULT_ENCRYPTION_KEY = 'default-key-change-in-production';

const normalizeHeaders = (target: SecurityTestTarget): Record<string, string> => {
  const mergedHeaders: Record<string, string> = {};
  const assignHeader = (key: string | null, value: string | null) => {
    if (!key || !value) return;
    mergedHeaders[key.toLowerCase()] = value;
  };

  if (typeof document !== 'undefined') {
    document.head.querySelectorAll('meta').forEach((meta) => {
      assignHeader(meta.getAttribute('http-equiv'), meta.getAttribute('content'));
      assignHeader(meta.getAttribute('name'), meta.getAttribute('content'));
    });
  }

  Object.entries(target.headers ?? {}).forEach(([key, value]) => assignHeader(key, value));
  return mergedHeaders;
};

const resolveCsp = (target: SecurityTestTarget, headers: Record<string, string>): string =>
  target.csp ?? headers['content-security-policy'] ?? DEFAULT_CSP;

const isSecureEndpoint = (endpoint: string): boolean =>
  endpoint.startsWith('https://') ||
  endpoint.startsWith('http://localhost') ||
  endpoint.startsWith('http://127.0.0.1');

const buildFinding = (
  scenario: SecurityScenario,
  finding: Omit<SecurityFinding, 'id' | 'title' | 'category' | 'severity' | 'control'>
): SecurityFinding => ({
  id: scenario.id,
  title: scenario.name,
  category: scenario.type,
  severity: scenario.severity,
  control: scenario.control,
  ...finding,
});

export const penetrationTestScenarios: SecurityScenario[] = [
  {
    id: 'penetration-xss-html',
    name: 'XSS payload sanitization',
    description: 'Dangerous HTML payloads should be neutralized before rendering.',
    type: 'penetration',
    severity: 'critical',
    control: 'OWASP ASVS 5.1.3',
    run(target) {
      const payloads = target.htmlPayloads ?? DEFAULT_HTML_PAYLOADS;
      const vulnerablePayloads = payloads.filter((payload) => DANGEROUS_PATTERN.test(sanitizeHtml(payload)));

      return {
        passed: vulnerablePayloads.length === 0,
        description:
          vulnerablePayloads.length === 0
            ? `Sanitized ${payloads.length} HTML attack payloads without executable markup surviving.`
            : 'One or more HTML payloads retained executable content after sanitization.',
        remediation: 'Sanitize all untrusted HTML with a strict allowlist before rendering user-controlled content.',
        metadata: {
          checkedPayloads: payloads.length,
          vulnerablePayloads,
        },
      };
    },
  },
  {
    id: 'penetration-dom-xss',
    name: 'DOM XSS output encoding',
    description: 'Injected markup should be escaped before insertion into the DOM.',
    type: 'penetration',
    severity: 'high',
    control: 'OWASP ASVS 5.3.2',
    run(target) {
      const payloads = target.htmlPayloads ?? DEFAULT_HTML_PAYLOADS;
      const failedPayloads = payloads.filter((payload) => {
        const encoded = XSSProtection.preventXSS(payload);
        return encoded.includes('<') || encoded.includes('>');
      });

      return {
        passed: failedPayloads.length === 0,
        description:
          failedPayloads.length === 0
            ? `Escaped ${payloads.length} DOM payloads without leaving raw tag delimiters behind.`
            : 'Some payloads still contain raw markup after DOM XSS encoding.',
        remediation: 'Escape untrusted text with `textContent` or equivalent output encoding before DOM insertion.',
        metadata: {
          checkedPayloads: payloads.length,
          failedPayloads,
        },
      };
    },
  },
  {
    id: 'penetration-attribute-injection',
    name: 'Attribute injection defense',
    description: 'Dangerous attributes and javascript URLs should be stripped from user input.',
    type: 'penetration',
    severity: 'high',
    control: 'OWASP ASVS 5.1.4',
    run(target) {
      const payloads = target.attributePayloads ?? DEFAULT_ATTRIBUTE_PAYLOADS;
      const failedPayloads = payloads.filter((payload) => DANGEROUS_PATTERN.test(XSSProtection.sanitizeAttribute(payload)));

      return {
        passed: failedPayloads.length === 0,
        description:
          failedPayloads.length === 0
            ? `Sanitized ${payloads.length} dangerous attribute payloads.`
            : 'Attribute sanitization still allows executable content.',
        remediation: 'Remove event handlers, script protocols, and unsafe delimiters from attribute values.',
        metadata: {
          checkedPayloads: payloads.length,
          failedPayloads,
        },
      };
    },
  },
  {
    id: 'penetration-csrf',
    name: 'CSRF token validation',
    description: 'Invalid CSRF tokens should be rejected while valid ones continue to work.',
    type: 'penetration',
    severity: 'high',
    control: 'OWASP ASVS 4.3.2',
    run(target) {
      const validToken = CSRFProtection.generateToken();
      const invalidToken = target.csrf?.invalidToken ?? `${validToken}-tampered`;

      const validAccepted = CSRFProtection.validateToken(validToken);
      const invalidRejected = !CSRFProtection.validateToken(invalidToken);

      return {
        passed: validAccepted && invalidRejected,
        description:
          validAccepted && invalidRejected
            ? 'CSRF validation accepts the current session token and rejects tampered tokens.'
            : 'CSRF validation does not clearly distinguish trusted and tampered tokens.',
        remediation: 'Validate a per-session anti-CSRF token on every state-changing request.',
        metadata: {
          validAccepted,
          invalidRejected,
        },
      };
    },
  },
  {
    id: 'penetration-rate-limit',
    name: 'Brute-force rate limiting',
    description: 'Repeated requests should be throttled after the configured threshold.',
    type: 'penetration',
    severity: 'medium',
    control: 'OWASP ASVS 7.2.1',
    run(target) {
      const maxAttempts = target.rateLimit?.maxAttempts ?? 3;
      const attempts = target.rateLimit?.attempts ?? maxAttempts + 1;
      const limiter = new RateLimiter(maxAttempts, target.rateLimit?.windowMs ?? 1000);
      const key = target.rateLimit?.key ?? 'security-assessment';

      let blocked = false;
      for (let index = 0; index < attempts; index += 1) {
        if (!limiter.isAllowed(key)) {
          blocked = true;
          break;
        }
      }

      return {
        passed: blocked,
        description: blocked
          ? `Rate limiting blocked repeated access after ${maxAttempts} allowed attempts.`
          : 'Rate limiting did not throttle repeated requests as expected.',
        remediation: 'Apply per-actor throttling for authentication, token, and high-risk workflow endpoints.',
        metadata: {
          maxAttempts,
          attemptsTested: attempts,
          remainingAttempts: limiter.getRemainingAttempts(key),
        },
      };
    },
  },
  {
    id: 'penetration-storage-confidentiality',
    name: 'Secure storage confidentiality',
    description: 'Sensitive browser storage entries should not be persisted in plaintext.',
    type: 'penetration',
    severity: 'high',
    control: 'OWASP ASVS 9.1.1',
    run(target) {
      const cases = target.secureStorageCases ?? [{ key: 'session', value: { token: 'secret' }, encrypt: true }];

      const failedCases = cases.filter(({ key, value, encrypt = true }) => {
        secureStorage.setItem(key, value, encrypt);
        const storedValue = localStorage.getItem(`stellar_escrow_${key}`);
        const restoredValue = secureStorage.getItem(key, encrypt);
        secureStorage.removeItem(key);

        return storedValue === JSON.stringify(value) || JSON.stringify(restoredValue) !== JSON.stringify(value);
      });

      return {
        passed: failedCases.length === 0,
        description:
          failedCases.length === 0
            ? `Verified encrypted persistence and recovery for ${cases.length} browser storage cases.`
            : 'One or more stored values were exposed in plaintext or could not be safely restored.',
        remediation: 'Encrypt sensitive browser storage and verify decrypted payload integrity before use.',
        metadata: {
          checkedCases: cases.length,
          failedKeys: failedCases.map((item) => item.key),
        },
      };
    },
  },
];

export const vulnerabilityScanScenarios: SecurityScenario[] = [
  {
    id: 'vulnerability-secure-endpoints',
    name: 'Secure transport endpoints',
    description: 'External endpoints should use HTTPS unless explicitly local.',
    type: 'vulnerability',
    severity: 'high',
    control: 'PCI DSS 4.0 4.2.1',
    run(target) {
      const endpoints = target.endpoints ?? ['https://api.stellar.org'];
      const insecureEndpoints = endpoints.filter((endpoint) => !isSecureEndpoint(endpoint));

      return {
        passed: insecureEndpoints.length === 0,
        description:
          insecureEndpoints.length === 0
            ? `Checked ${endpoints.length} endpoints and found no insecure transport usage.`
            : 'One or more endpoints still use insecure transport.',
        remediation: 'Require HTTPS for all production APIs, asset origins, and callback URLs.',
        metadata: {
          checkedEndpoints: endpoints.length,
          insecureEndpoints,
        },
      };
    },
  },
  {
    id: 'vulnerability-csp-hardening',
    name: 'CSP hardening',
    description: 'Content Security Policy should block wildcards, unsafe-eval, and missing framing controls.',
    type: 'vulnerability',
    severity: 'high',
    control: 'OWASP ASVS 14.4.3',
    run(target) {
      const headers = normalizeHeaders(target);
      const csp = resolveCsp(target, headers);
      const issues: string[] = [];

      if (!csp.includes("frame-ancestors 'none'")) {
        issues.push('missing frame-ancestors');
      }
      if (!csp.includes("base-uri 'self'")) {
        issues.push('missing base-uri');
      }
      if (!csp.includes("form-action 'self'")) {
        issues.push('missing form-action');
      }
      if (/\*/.test(csp)) {
        issues.push('wildcard source');
      }
      if (/'unsafe-eval'/.test(csp)) {
        issues.push('unsafe-eval enabled');
      }

      return {
        passed: issues.length === 0,
        description:
          issues.length === 0
            ? 'CSP contains baseline anti-framing and anti-script-injection controls.'
            : 'CSP contains missing or weak directives that increase exploitability.',
        remediation: 'Tighten CSP directives and remove wildcard or unsafe-eval allowances in production.',
        metadata: {
          csp,
          issues,
        },
      };
    },
  },
  {
    id: 'vulnerability-security-headers',
    name: 'Security header coverage',
    description: 'Baseline browser hardening headers should be configured.',
    type: 'vulnerability',
    severity: 'medium',
    control: 'OWASP ASVS 14.4.1',
    run(target) {
      const headers = normalizeHeaders(target);
      const missingHeaders = Object.entries(DEFAULT_SECURITY_HEADERS)
        .filter(([key]) => !headers[key])
        .map(([key]) => key);

      return {
        passed: missingHeaders.length === 0,
        description:
          missingHeaders.length === 0
            ? 'Security headers cover browser sniffing, framing, referrer, permissions, and XSS protections.'
            : 'Required browser hardening headers are missing.',
        remediation: 'Set X-Content-Type-Options, X-Frame-Options, Referrer-Policy, Permissions-Policy, and X-XSS-Protection.',
        metadata: {
          missingHeaders,
          configuredHeaders: headers,
        },
      };
    },
  },
  {
    id: 'vulnerability-encryption-key',
    name: 'Encryption key hygiene',
    description: 'Sensitive data should not rely on a default placeholder encryption key.',
    type: 'vulnerability',
    severity: 'critical',
    control: 'OWASP ASVS 6.2.6',
    run(target) {
      const encryptionKey = target.encryptionKey ?? DEFAULT_ENCRYPTION_KEY;
      const usesDefaultKey = encryptionKey === DEFAULT_ENCRYPTION_KEY;
      const tooShort = encryptionKey.length < 16;

      return {
        passed: !usesDefaultKey && !tooShort,
        description:
          !usesDefaultKey && !tooShort
            ? 'Encryption key configuration is non-default and meets the minimum length baseline.'
            : 'Encryption key configuration is weak or still using the default placeholder.',
        remediation: 'Inject a unique production encryption key from secure secret management with adequate entropy.',
        metadata: {
          usesDefaultKey,
          keyLength: encryptionKey.length,
        },
      };
    },
  },
];

export const complianceTestScenarios: SecurityScenario[] = [
  {
    id: 'compliance-owasp-browser-controls',
    name: 'OWASP browser defense baseline',
    description: 'Browser-facing security controls should meet an OWASP-style baseline.',
    type: 'compliance',
    severity: 'high',
    control: 'OWASP ASVS Baseline',
    run(target) {
      const headers = normalizeHeaders(target);
      const csp = resolveCsp(target, headers);
      const missingControls = [
        !headers['x-content-type-options'] && 'x-content-type-options',
        !headers['x-frame-options'] && 'x-frame-options',
        !headers['x-xss-protection'] && 'x-xss-protection',
        !headers['referrer-policy'] && 'referrer-policy',
        !headers['permissions-policy'] && 'permissions-policy',
        !csp.includes("frame-ancestors 'none'") && 'frame-ancestors',
      ].filter(Boolean) as string[];

      return {
        passed: missingControls.length === 0,
        description:
          missingControls.length === 0
            ? 'OWASP-aligned browser controls are present for framing, sniffing, policy, and script handling.'
            : 'OWASP-aligned browser control requirements are incomplete.',
        remediation: 'Close browser-defense gaps before release and keep these controls in deployment policy checks.',
        metadata: {
          missingControls,
        },
      };
    },
  },
  {
    id: 'compliance-pci-transport',
    name: 'PCI transport protection',
    description: 'Payment-adjacent data paths should enforce encrypted transport.',
    type: 'compliance',
    severity: 'high',
    control: 'PCI DSS 4.0 4.2',
    run(target) {
      const endpoints = target.endpoints ?? ['https://api.stellar.org'];
      const insecureEndpoints = endpoints.filter((endpoint) => !isSecureEndpoint(endpoint));
      const httpsEnforced = target.requireHttps ?? true;

      return {
        passed: httpsEnforced && insecureEndpoints.length === 0,
        description:
          httpsEnforced && insecureEndpoints.length === 0
            ? 'Encrypted transport is enforced for configured network paths.'
            : 'Transport controls do not fully satisfy encrypted-channel requirements.',
        remediation: 'Redirect all production traffic to HTTPS and disallow insecure callback or API origins.',
        metadata: {
          httpsEnforced,
          insecureEndpoints,
        },
      };
    },
  },
  {
    id: 'compliance-monitoring-readiness',
    name: 'Security monitoring readiness',
    description: 'Security events should be observable and routable to a monitoring endpoint.',
    type: 'compliance',
    severity: 'medium',
    control: 'NIST AU-6',
    run(target) {
      const enabled = target.monitoring?.enabled ?? false;
      const endpoint = target.monitoring?.endpoint ?? '';

      return {
        passed: enabled && endpoint.length > 0,
        description:
          enabled && endpoint.length > 0
            ? 'Security monitoring is enabled with a defined reporting endpoint.'
            : 'Security monitoring is not fully configured for alert forwarding.',
        remediation: 'Enable security telemetry and ship alerts to a monitored endpoint before production rollout.',
        metadata: {
          enabled,
          endpoint,
        },
      };
    },
  },
];

export const defaultSecurityScenarios: SecurityScenario[] = [
  ...penetrationTestScenarios,
  ...vulnerabilityScanScenarios,
  ...complianceTestScenarios,
];

export const runSecurityScenarios = (
  target: SecurityTestTarget,
  scenarios: SecurityScenario[],
  monitor?: SecurityMonitor
): SecurityFinding[] =>
  scenarios.map((scenario) => {
    const finding = buildFinding(scenario, scenario.run(target));
    monitor?.recordFinding(finding);
    return finding;
  });
