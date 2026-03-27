# Security Implementation

Comprehensive security utilities for XSS protection, CSP, secure storage, and input validation.

## Features

### ✅ Content Security Policy
- Meta tag-based CSP configuration
- Strict default-src policy
- Whitelisted external resources
- Frame-ancestors protection

### ✅ XSS Protection
- HTML sanitization with DOMPurify
- Input sanitization
- HTML escaping
- Script tag removal
- Clickjacking prevention
- Frame injection prevention

### ✅ Secure Storage
- AES encryption for sensitive data
- Prefixed localStorage keys
- Automatic encryption/decryption
- Secure data retrieval

### ✅ Input Sanitization
- HTML sanitization
- Input cleaning
- Email validation
- URL validation
- Stellar address validation
- Pattern validation

### ✅ Security Headers
- X-Content-Type-Options
- X-Frame-Options
- X-XSS-Protection
- Referrer-Policy
- Permissions-Policy

### ✅ Security Testing
- Penetration testing scenarios
- Vulnerability scanning
- Compliance control checks
- Security event monitoring
- Assessment summaries with severity breakdowns

## Usage

### Initialize Security
```tsx
import { initializeSecurity } from '@stellar-escrow/security';

// Call at app startup
initializeSecurity();
```

### Sanitize Input
```tsx
import { sanitizeInput, sanitizeHtml, escapeHtml } from '@stellar-escrow/security';

const userInput = sanitizeInput(input);
const htmlContent = sanitizeHtml(html);
const escaped = escapeHtml(text);
```

### Secure Storage
```tsx
import { secureStorage } from '@stellar-escrow/security';

// Store encrypted data
secureStorage.setItem('token', { value: 'secret' }, true);

// Retrieve and decrypt
const data = secureStorage.getItem('token', true);

// Clear all
secureStorage.clear();
```

### XSS Protection
```tsx
import { XSSProtection, CSRFProtection } from '@stellar-escrow/security';

// Prevent XSS
const safe = XSSProtection.preventXSS(html);

// CSRF token
const token = CSRFProtection.generateToken();
const headers = CSRFProtection.setTokenHeader({});
```

### Input Validation
```tsx
import { InputValidator, RateLimiter } from '@stellar-escrow/security';

// Validate input
InputValidator.validateLength(input, 1, 100);
InputValidator.validateDecimal(amount, 2);
InputValidator.validateAlphanumeric(username);

// Rate limiting
const limiter = new RateLimiter(5, 60000);
if (limiter.isAllowed('user-id')) {
  // Allow request
}
```

### Security Assessment
```tsx
import { runSecurityAssessment, SecurityMonitor } from '@stellar-escrow/security';

const monitor = new SecurityMonitor({ endpoint: '/api/security/alerts' });

const result = runSecurityAssessment(
  {
    endpoints: ['https://api.stellar.org'],
    encryptionKey: process.env.REACT_APP_ENCRYPTION_KEY,
    monitoring: { enabled: true, endpoint: '/api/security/alerts' },
  },
  monitor
);

console.log(result.overallPassed, result.totals, monitor.getAlerts());
```

## Security Measures

### CSP Configuration
```
default-src 'self'
script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com
font-src 'self' https://fonts.gstatic.com
img-src 'self' data: https:
connect-src 'self' https://api.stellar.org
frame-ancestors 'none'
base-uri 'self'
form-action 'self'
```

### Encryption
- AES encryption for sensitive data
- Configurable encryption key
- Automatic serialization/deserialization

### Input Validation
- Length validation
- Pattern matching
- Special character detection
- Format validation (email, URL, address)

### Rate Limiting
- Configurable max attempts
- Time window-based limiting
- Per-key tracking
- Remaining attempts calculation

## Files

- `sanitization.ts` - HTML/input sanitization
- `storage.ts` - Secure encrypted storage
- `headers.ts` - Security headers setup
- `xss.ts` - XSS and CSRF protection
- `validation.ts` - Input validation and rate limiting
- `index.ts` - Main export
- `security.test.ts` - Tests

## Testing

```bash
npm test
```

Tests cover:
- HTML sanitization
- Input sanitization
- Encryption/decryption
- XSS prevention
- CSRF token generation
- Rate limiting
- Input validation
- Penetration scenarios for XSS, CSRF, brute force, and storage confidentiality
- Vulnerability scans for CSP, transport security, header coverage, and key hygiene
- Compliance checks for OWASP, PCI-style transport controls, and monitoring readiness
- Security monitoring and beacon-based alert reporting

### Run The Security Suite

```bash
npm run test:security --workspace=security
```

## Best Practices

1. Always sanitize user input
2. Use secure storage for sensitive data
3. Validate all inputs
4. Implement rate limiting
5. Use CSRF tokens for state-changing operations
6. Enable CSP headers
7. Escape HTML output
8. Validate email and addresses

## Dependencies

- dompurify: ^3.0.6 - HTML sanitization
- crypto-js: ^4.1.1 - Encryption

## Next Steps

1. Integrate with React components
2. Add more validation rules
3. Implement audit logging
4. Add security monitoring
5. Set up security headers on backend
