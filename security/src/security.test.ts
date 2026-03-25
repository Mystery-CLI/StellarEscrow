import { sanitizeHtml, sanitizeInput, escapeHtml, validateEmail, validateAddress } from '../sanitization';
import { secureStorage } from '../storage';
import { XSSProtection, CSRFProtection } from '../xss';
import { RateLimiter, InputValidator } from '../validation';

describe('Security', () => {
  it('should sanitize HTML', () => {
    const dirty = '<img src=x onerror="alert(1)">';
    const clean = sanitizeHtml(dirty);
    expect(clean).not.toContain('onerror');
  });

  it('should sanitize input', () => {
    const input = '<script>alert(1)</script>';
    const clean = sanitizeInput(input);
    expect(clean).not.toContain('<');
    expect(clean).not.toContain('>');
  });

  it('should escape HTML', () => {
    const html = '<div>test</div>';
    const escaped = escapeHtml(html);
    expect(escaped).toBe('&lt;div&gt;test&lt;/div&gt;');
  });

  it('should validate email', () => {
    expect(validateEmail('test@example.com')).toBe(true);
    expect(validateEmail('invalid')).toBe(false);
  });

  it('should validate Stellar address', () => {
    const validAddress = 'GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE';
    expect(validateAddress(validAddress)).toBe(true);
    expect(validateAddress('invalid')).toBe(false);
  });

  it('should store and retrieve encrypted data', () => {
    secureStorage.setItem('test', { data: 'secret' }, true);
    const retrieved = secureStorage.getItem('test', true);
    expect(retrieved).toEqual({ data: 'secret' });
  });

  it('should prevent XSS', () => {
    const html = '<img src=x onerror="alert(1)">';
    const safe = XSSProtection.preventXSS(html);
    expect(safe).not.toContain('onerror');
  });

  it('should generate CSRF token', () => {
    const token = CSRFProtection.generateToken();
    expect(token).toBeDefined();
    expect(CSRFProtection.validateToken(token)).toBe(true);
  });

  it('should rate limit requests', () => {
    const limiter = new RateLimiter(2, 1000);
    expect(limiter.isAllowed('user1')).toBe(true);
    expect(limiter.isAllowed('user1')).toBe(true);
    expect(limiter.isAllowed('user1')).toBe(false);
  });

  it('should validate input length', () => {
    expect(InputValidator.validateLength('test', 1, 10)).toBe(true);
    expect(InputValidator.validateLength('test', 5, 10)).toBe(false);
  });

  it('should validate decimal', () => {
    expect(InputValidator.validateDecimal('100.50', 2)).toBe(true);
    expect(InputValidator.validateDecimal('100.999', 2)).toBe(false);
  });
});
