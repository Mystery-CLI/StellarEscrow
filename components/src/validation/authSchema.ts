import { isValidStellarAddress, type ValidationRule } from './tradeSchema';

/**
 * Validation schema for Authentication (Login/Register) form fields.
 */

export type AuthFieldName = 'address' | 'usernameHash' | 'contactHash';

export type AuthFieldSchema = ValidationRule[];

export const isValidHash = (value: string): boolean => {
  return /^[a-fA-F0-9]{64}$/.test(value.trim());
};

export const authSchema: Record<AuthFieldName, AuthFieldSchema> = {
  address: [
    {
      test: (v) => v.trim().length > 0,
      message: 'Stellar address is required',
    },
    {
      test: (v) => isValidStellarAddress(v),
      message: 'Must be a valid Stellar address (G… 56 chars)',
    },
  ],

  usernameHash: [
    {
      test: (v) => v.trim().length > 0,
      message: 'Username hash is required',
    },
    {
      test: (v) => isValidHash(v),
      message: 'Must be a valid 64-character SHA-256 hex hash',
    },
  ],

  contactHash: [
    {
      test: (v) => v.trim().length > 0,
      message: 'Contact hash is required',
    },
    {
      test: (v) => isValidHash(v),
      message: 'Must be a valid 64-character SHA-256 hex hash',
    },
  ],
};

/**
 * Returns the first failing rule's message, or null if valid.
 */
export function validateAuthField(
  field: AuthFieldName,
  value: string
): string | null {
  const schema = authSchema[field];
  if (!schema) return null;
  for (const rule of schema) {
    if (!rule.test(value)) return rule.message;
  }
  return null;
}

/**
 * Returns true only if the field passes all rules.
 */
export function isAuthFieldValid(
  field: AuthFieldName,
  value: string
): boolean {
  return validateAuthField(field, value) === null && value.trim().length > 0;
}
