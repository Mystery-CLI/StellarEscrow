/**
 * A secure wrapper for localStorage that provides type safety, JSON parsing protection,
 * and basic checks to prevent accidental storage of sensitive data (like secret keys).
 */
export const secureStorage = {
  /**
   * Safe setItem that stringifies data and warns against sensitive key names.
   */
  setItem: (key: string, value: any): void => {
    // SECURITY PRACTICE: Avoid storing high-sensitivity data in plain text in localStorage.
    // If you must store sensitive data, it should be encrypted first.
    if (key.toLowerCase().includes('secret') || key.toLowerCase().includes('key')) {
      console.warn(`Security Warning: Potentially sensitive data being stored under key: ${key}`);
    }
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch (e) {
      console.error(`SecureStorage Error: Failed to save ${key}`, e);
    }
  },

  /**
   * Safe getItem that parses JSON and returns null if parsing fails.
   */
  getItem: <T>(key: string): T | null => {
    const item = localStorage.getItem(key);
    if (!item) return null;
    try {
      // JSON parsing safety: avoids runtime errors if the value is not valid JSON
      return JSON.parse(item) as T;
    } catch (e) {
      console.error(`SecureStorage Error: Failed to parse item ${key}`, e);
      return null;
    }
  },

  /**
   * Safe removeItem wrapper.
   */
  removeItem: (key: string): void => {
    localStorage.removeItem(key);
  },

  /**
   * Clears all localStorage items.
   */
  clear: (): void => {
    localStorage.clear();
  }
};
