import {
  isValidStellarAddress,
  isPositiveNumber,
  validateField,
  isFieldValid,
  validateTradeForm,
} from './tradeSchema';

const VALID_ADDR_A = 'GBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQP';
const VALID_ADDR_B = 'GGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM';

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

describe('isValidStellarAddress', () => {
  it('accepts a valid G-address', () => {
    expect(isValidStellarAddress(VALID_ADDR_A)).toBe(true);
  });

  it('rejects address not starting with G', () => {
    expect(isValidStellarAddress('SAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN')).toBe(false);
  });

  it('rejects address shorter than 56 chars', () => {
    expect(isValidStellarAddress('GABC')).toBe(false);
  });

  it('rejects empty string', () => {
    expect(isValidStellarAddress('')).toBe(false);
  });

  it('rejects address with lowercase letters', () => {
    expect(isValidStellarAddress('gaazi4tcr3ty5ojhctjc2a4qsy6cjwjh5iajtgkin2er7lbnvkoccwn')).toBe(false);
  });
});

describe('isPositiveNumber', () => {
  it('accepts positive integer', () => expect(isPositiveNumber('100')).toBe(true));
  it('accepts positive decimal', () => expect(isPositiveNumber('0.001')).toBe(true));
  it('rejects zero', () => expect(isPositiveNumber('0')).toBe(false));
  it('rejects negative', () => expect(isPositiveNumber('-5')).toBe(false));
  it('rejects non-numeric', () => expect(isPositiveNumber('abc')).toBe(false));
  it('rejects empty string', () => expect(isPositiveNumber('')).toBe(false));
});

// ---------------------------------------------------------------------------
// validateField
// ---------------------------------------------------------------------------

describe('validateField — seller', () => {
  it('returns error for empty value', () => {
    expect(validateField('seller', '')).toBe('Seller address is required');
  });

  it('returns error for invalid address', () => {
    expect(validateField('seller', 'G123')).toMatch(/valid Stellar address/i);
  });

  it('returns null for valid address', () => {
    expect(validateField('seller', VALID_ADDR_A)).toBeNull();
  });
});

describe('validateField — buyer', () => {
  it('returns error when buyer equals seller', () => {
    const err = validateField('buyer', VALID_ADDR_A, { seller: VALID_ADDR_A });
    expect(err).toMatch(/different/i);
  });

  it('returns null when buyer differs from seller', () => {
    expect(validateField('buyer', VALID_ADDR_B, { seller: VALID_ADDR_A })).toBeNull();
  });
});

describe('validateField — amount', () => {
  it('returns error for empty', () => expect(validateField('amount', '')).toBe('Amount is required'));
  it('returns error for zero', () => expect(validateField('amount', '0')).toMatch(/positive/i));
  it('returns error for negative', () => expect(validateField('amount', '-1')).toMatch(/positive/i));
  it('returns null for valid amount', () => expect(validateField('amount', '100')).toBeNull());
  it('returns error for amount exceeding max', () => {
    expect(validateField('amount', '2000000000')).toMatch(/maximum/i);
  });
});

describe('validateField — arbitrator', () => {
  it('returns null for empty (optional)', () => {
    expect(validateField('arbitrator', '')).toBeNull();
  });

  it('returns error for non-empty invalid address', () => {
    expect(validateField('arbitrator', 'bad')).toMatch(/valid Stellar address/i);
  });

  it('returns null for valid address', () => {
    expect(validateField('arbitrator', VALID_ADDR_A)).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// isFieldValid
// ---------------------------------------------------------------------------

describe('isFieldValid', () => {
  it('returns true for valid seller', () => {
    expect(isFieldValid('seller', VALID_ADDR_A)).toBe(true);
  });

  it('returns false for invalid seller', () => {
    expect(isFieldValid('seller', 'bad')).toBe(false);
  });

  it('returns false for empty arbitrator (optional — neutral)', () => {
    expect(isFieldValid('arbitrator', '')).toBe(false);
  });

  it('returns true for valid arbitrator', () => {
    expect(isFieldValid('arbitrator', VALID_ADDR_A)).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// validateTradeForm
// ---------------------------------------------------------------------------

describe('validateTradeForm', () => {
  it('returns errors for all empty fields', () => {
    const errors = validateTradeForm({ seller: '', buyer: '', amount: '', arbitrator: '' });
    expect(errors.seller).toBeDefined();
    expect(errors.buyer).toBeDefined();
    expect(errors.amount).toBeDefined();
    expect(errors.arbitrator).toBeUndefined(); // optional
  });

  it('returns no errors for a fully valid form', () => {
    const errors = validateTradeForm({
      seller: VALID_ADDR_A,
      buyer: VALID_ADDR_B,
      amount: '100',
      arbitrator: '',
    });
    expect(Object.keys(errors)).toHaveLength(0);
  });

  it('catches buyer === seller cross-field error', () => {
    const errors = validateTradeForm({
      seller: VALID_ADDR_A,
      buyer: VALID_ADDR_A,
      amount: '100',
      arbitrator: '',
    });
    expect(errors.buyer).toMatch(/different/i);
  });
});
