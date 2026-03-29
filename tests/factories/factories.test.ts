import {
  userFactory, userList,
  escrowFactory, escrowList, resetEscrowSeq,
  assetFactory, USDC_ASSET,
  stellarAddress,
} from '../factories';

// ── Stellar address ──────────────────────────────────────────────────────────

describe('stellarAddress', () => {
  it('produces a 56-char G-address', () => {
    const addr = stellarAddress();
    expect(addr).toHaveLength(56);
    expect(addr[0]).toBe('G');
  });

  it('produces unique addresses', () => {
    const addrs = new Set(Array.from({ length: 100 }, stellarAddress));
    expect(addrs.size).toBe(100);
  });

  it('uses only valid base32 chars', () => {
    const valid = /^[A-Z2-7]{56}$/;
    expect(stellarAddress()).toMatch(valid);
  });
});

// ── User factory ─────────────────────────────────────────────────────────────

describe('userFactory', () => {
  it('generates a valid user', () => {
    const u = userFactory();
    expect(u.address).toHaveLength(56);
    expect(u.display_name).toBeTruthy();
    expect(typeof u.kyc_verified).toBe('boolean');
    expect(u.created_at).toBeInstanceOf(Date);
  });

  it('respects overrides', () => {
    const u = userFactory({ kyc_verified: true });
    expect(u.kyc_verified).toBe(true);
  });

  it('userList produces N distinct addresses', () => {
    const users = userList(5);
    expect(users).toHaveLength(5);
    const addrs = new Set(users.map((u) => u.address));
    expect(addrs.size).toBe(5);
  });
});

// ── Escrow factory ───────────────────────────────────────────────────────────

describe('escrowFactory', () => {
  beforeEach(() => resetEscrowSeq(1));

  it('generates a valid escrow', () => {
    const e = escrowFactory();
    expect(e.trade_id).toBe(1);
    expect(e.seller).toHaveLength(56);
    expect(e.buyer).toHaveLength(56);
    expect(e.amount).toBeGreaterThanOrEqual(1_000_000);
    expect(e.fee).toBe(Math.floor(e.amount * 0.01));
    expect(['created', 'funded', 'completed', 'disputed', 'cancelled']).toContain(e.status);
  });

  it('seller and buyer are different addresses', () => {
    // Statistically guaranteed with 56-char random addresses
    const e = escrowFactory();
    expect(e.seller).not.toBe(e.buyer);
  });

  it('escrowList cycles through all 5 statuses', () => {
    const escrows = escrowList(10);
    const statuses = new Set(escrows.map((e) => e.status));
    expect(statuses.size).toBe(5);
  });

  it('trade_id increments sequentially', () => {
    const [a, b, c] = escrowList(3);
    expect(a.trade_id).toBe(1);
    expect(b.trade_id).toBe(2);
    expect(c.trade_id).toBe(3);
  });

  it('fee is 1% of amount', () => {
    const e = escrowFactory({ amount: 10_000_000 });
    expect(e.fee).toBe(100_000);
  });
});

// ── Asset factory ────────────────────────────────────────────────────────────

describe('assetFactory', () => {
  it('generates a valid asset', () => {
    const a = assetFactory();
    expect(a.code).toBeTruthy();
    expect(a.decimals).toBe(7);
  });

  it('USDC_ASSET is not native and has a valid issuer', () => {
    expect(USDC_ASSET.code).toBe('USDC');
    expect(USDC_ASSET.is_native).toBe(false);
    expect(USDC_ASSET.issuer).toHaveLength(56);
  });

  it('XLM asset is native with issuer=native', () => {
    const xlm = assetFactory({ code: 'XLM', is_native: true, issuer: 'native' });
    expect(xlm.is_native).toBe(true);
    expect(xlm.issuer).toBe('native');
  });
});

// ── Privacy compliance ───────────────────────────────────────────────────────

describe('PII compliance', () => {
  it('user display_name contains no real email patterns', () => {
    const users = userList(20);
    for (const u of users) {
      // Faker usernames should not look like real email addresses
      expect(u.display_name).not.toMatch(/@.*\.(com|org|net)$/i);
    }
  });

  it('no factory field contains a real phone number pattern', () => {
    const users = userList(10);
    const phonePattern = /\+?[\d\s\-().]{10,}/;
    for (const u of users) {
      expect(u.display_name).not.toMatch(phonePattern);
    }
  });
});
