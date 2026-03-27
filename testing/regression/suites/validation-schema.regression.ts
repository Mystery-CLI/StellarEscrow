/**
 * Validation Schema Regression Suite
 *
 * Ensures the trade form validation rules haven't regressed.
 * These are pure-function tests — no network required.
 */

import {
  isValidStellarAddress,
  isPositiveNumber,
  validateField,
  validateTradeForm,
} from '../../../components/src/validation/tradeSchema';

const ADDR_A = 'GBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQP';
const ADDR_B = 'GGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM';

describe('Validation Regression — Stellar address format', () => {
  const valid = [ADDR_A, ADDR_B];
  const invalid = ['', 'G123', 'SAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN', 'g' + ADDR_A.slice(1)];

  valid.forEach(addr => {
    it(`accepts valid address: ${addr.slice(0, 8)}...`, () => {
      expect(isValidStellarAddress(addr)).toBe(true);
    });
  });

  invalid.forEach(addr => {
    it(`rejects invalid address: "${addr.slice(0, 12) || '(empty)'}"`, () => {
      expect(isValidStellarAddress(addr)).toBe(false);
    });
  });
});

describe('Validation Regression — Amount rules', () => {
  const valid = ['1', '0.001', '1000000', '999999999'];
  const invalid = ['0', '-1', 'abc', '', '2000000000'];

  valid.forEach(v => {
    it(`accepts amount: ${v}`, () => {
      expect(isPositiveNumber(v)).toBe(true);
    });
  });

  invalid.forEach(v => {
    it(`rejects amount: "${v || '(empty)'}"`, () => {
      if (v === '2000000000') {
        // max limit check is in validateField, not isPositiveNumber
        expect(validateField('amount', v)).toMatch(/maximum/i);
      } else {
        expect(isPositiveNumber(v)).toBe(false);
      }
    });
  });
});

describe('Validation Regression — Cross-field rules', () => {
  it('buyer cannot equal seller', () => {
    const err = validateField('buyer', ADDR_A, { seller: ADDR_A });
    expect(err).toMatch(/different/i);
  });

  it('buyer can differ from seller', () => {
    expect(validateField('buyer', ADDR_B, { seller: ADDR_A })).toBeNull();
  });

  it('arbitrator is optional — empty passes', () => {
    expect(validateField('arbitrator', '')).toBeNull();
  });

  it('arbitrator must be valid if provided', () => {
    expect(validateField('arbitrator', 'bad')).not.toBeNull();
    expect(validateField('arbitrator', ADDR_A)).toBeNull();
  });
});

describe('Validation Regression — Full form', () => {
  it('valid form produces no errors', () => {
    const errors = validateTradeForm({ seller: ADDR_A, buyer: ADDR_B, amount: '100', arbitrator: '' });
    expect(Object.keys(errors)).toHaveLength(0);
  });

  it('empty form produces errors for required fields only', () => {
    const errors = validateTradeForm({ seller: '', buyer: '', amount: '', arbitrator: '' });
    expect(errors.seller).toBeDefined();
    expect(errors.buyer).toBeDefined();
    expect(errors.amount).toBeDefined();
    expect(errors.arbitrator).toBeUndefined(); // optional
  });
});
