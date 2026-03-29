/**
 * Validation schema for trade form fields.
 *
 * Pure functions — no external deps, works in any environment.
 * Mirrors the Rust validation in ui/src/form.rs so both layers agree.
 */

export type FieldName = 'seller' | 'buyer' | 'amount' | 'arbitrator';

export interface ValidationRule {
  test: (value: string, ctx?: ValidationContext) => boolean;
  message: string;
}

export interface ValidationContext {
  seller?: string;
  buyer?: string;
}

export type FieldSchema = ValidationRule[];

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

/** Stellar public key: starts with 'G', exactly 56 base-32 chars */
export function isValidStellarAddress(value: string): boolean {
  return /^G[A-Z2-7]{55}$/.test(value.trim());
}

export function isPositiveNumber(value: string): boolean {
  const n = parseFloat(value.trim());
  return !isNaN(n) && isFinite(n) && n > 0;
}

// ---------------------------------------------------------------------------
// Field schemas
// ---------------------------------------------------------------------------

export const tradeSchema: Record<FieldName, FieldSchema> = {
  seller: [
    {
      test: (v) => v.trim().length > 0,
      message: 'Seller address is required',
    },
    {
      test: (v) => isValidStellarAddress(v),
      message: 'Must be a valid Stellar address (G… 56 chars)',
    },
  ],

  buyer: [
    {
      test: (v) => v.trim().length > 0,
      message: 'Buyer address is required',
    },
    {
      test: (v) => isValidStellarAddress(v),
      message: 'Must be a valid Stellar address (G… 56 chars)',
    },
    {
      test: (v, ctx) => !ctx?.seller || v.trim() !== ctx.seller.trim(),
      message: 'Buyer and seller must be different addresses',
    },
  ],

  amount: [
    {
      test: (v) => v.trim().length > 0,
      message: 'Amount is required',
    },
    {
      test: (v) => isPositiveNumber(v),
      message: 'Amount must be a positive number',
    },
    {
      test: (v) => {
        const n = parseFloat(v);
        return isNaN(n) || n <= 1_000_000_000;
      },
      message: 'Amount exceeds maximum allowed (1,000,000,000)',
    },
  ],

  arbitrator: [
    // Optional — only validate if non-empty
    {
      test: (v) => v.trim() === '' || isValidStellarAddress(v),
      message: 'Must be a valid Stellar address (G… 56 chars)',
    },
  ],
};

// ---------------------------------------------------------------------------
// Validate a single field
// ---------------------------------------------------------------------------

/**
 * Returns the first failing rule's message, or null if valid.
 */
export function validateField(
  field: FieldName,
  value: string,
  ctx?: ValidationContext
): string | null {
  for (const rule of tradeSchema[field]) {
    if (!rule.test(value, ctx)) return rule.message;
  }
  return null;
}

/**
 * Returns true only if the field has a non-empty value that passes all rules.
 * Used to show the green ✓ indicator.
 */
export function isFieldValid(
  field: FieldName,
  value: string,
  ctx?: ValidationContext
): boolean {
  if (field === 'arbitrator' && value.trim() === '') return false; // optional — neutral, not valid
  return validateField(field, value, ctx) === null && value.trim().length > 0;
}

// ---------------------------------------------------------------------------
// Validate entire form
// ---------------------------------------------------------------------------

export type FormErrors = Partial<Record<FieldName, string>>;

export function validateTradeForm(
  data: Record<FieldName, string>
): FormErrors {
  const ctx: ValidationContext = { seller: data.seller, buyer: data.buyer };
  const errors: FormErrors = {};

  for (const field of Object.keys(tradeSchema) as FieldName[]) {
    const error = validateField(field, data[field] ?? '', ctx);
    if (error) errors[field] = error;
  }

  return errors;
}
