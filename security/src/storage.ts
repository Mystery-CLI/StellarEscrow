import CryptoJS from 'crypto-js';

const DEFAULT_ENCRYPTION_KEY = 'default-key-change-in-production';

const resolveEncryptionKey = (): string => {
  const maybeProcess = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process;
  return maybeProcess?.env?.REACT_APP_ENCRYPTION_KEY || DEFAULT_ENCRYPTION_KEY;
};

export const encryptData = (data: string): string => {
  return CryptoJS.AES.encrypt(data, resolveEncryptionKey()).toString();
};

export const decryptData = (encrypted: string): string => {
  const bytes = CryptoJS.AES.decrypt(encrypted, resolveEncryptionKey());
  return bytes.toString(CryptoJS.enc.Utf8);
};

export const hashData = (data: string): string => {
  return CryptoJS.SHA256(data).toString();
};

export class SecureStorage {
  private prefix = 'stellar_escrow_';

  setItem(key: string, value: any, encrypt = true): void {
    const data = JSON.stringify(value);
    const encrypted = encrypt ? encryptData(data) : data;
    localStorage.setItem(this.prefix + key, encrypted);
  }

  getItem<T = any>(key: string, decrypt = true): T | null {
    const encrypted = localStorage.getItem(this.prefix + key);
    if (!encrypted) return null;
    try {
      const data = decrypt ? decryptData(encrypted) : encrypted;
      return JSON.parse(data);
    } catch {
      return null;
    }
  }

  removeItem(key: string): void {
    localStorage.removeItem(this.prefix + key);
  }

  clear(): void {
    Object.keys(localStorage).forEach((key) => {
      if (key.startsWith(this.prefix)) {
        localStorage.removeItem(key);
      }
    });
  }

  getAllKeys(): string[] {
    return Object.keys(localStorage)
      .filter((key) => key.startsWith(this.prefix))
      .map((key) => key.replace(this.prefix, ''));
  }
}

export const secureStorage = new SecureStorage();
