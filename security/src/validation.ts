export class RateLimiter {
  private attempts: Map<string, number[]> = new Map();
  private maxAttempts: number;
  private windowMs: number;

  constructor(maxAttempts = 5, windowMs = 60000) {
    this.maxAttempts = maxAttempts;
    this.windowMs = windowMs;
  }

  isAllowed(key: string): boolean {
    const now = Date.now();
    const attempts = this.attempts.get(key) || [];
    const recentAttempts = attempts.filter((time) => now - time < this.windowMs);

    if (recentAttempts.length >= this.maxAttempts) {
      return false;
    }

    recentAttempts.push(now);
    this.attempts.set(key, recentAttempts);
    return true;
  }

  reset(key: string): void {
    this.attempts.delete(key);
  }

  getRemainingAttempts(key: string): number {
    const now = Date.now();
    const attempts = this.attempts.get(key) || [];
    const recentAttempts = attempts.filter((time) => now - time < this.windowMs);
    return Math.max(0, this.maxAttempts - recentAttempts.length);
  }
}

export class InputValidator {
  static validateLength(input: string, min: number, max: number): boolean {
    return input.length >= min && input.length <= max;
  }

  static validatePattern(input: string, pattern: RegExp): boolean {
    return pattern.test(input);
  }

  static validateNoSpecialChars(input: string): boolean {
    return !/[<>{}[\]\\\/;:'"]/g.test(input);
  }

  static validateAlphanumeric(input: string): boolean {
    return /^[a-zA-Z0-9]+$/.test(input);
  }

  static validateNumeric(input: string): boolean {
    return /^\d+$/.test(input);
  }

  static validateDecimal(input: string, decimals = 2): boolean {
    const regex = new RegExp(`^\\d+(\\.\\d{1,${decimals}})?$`);
    return regex.test(input);
  }
}
