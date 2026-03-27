import { faker } from '@faker-js/faker';
import { stellarAddress, pick, randInt } from './stellar';

export type EscrowStatus = 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled';

export interface EscrowRecord {
  trade_id: number;
  seller: string;
  buyer: string;
  amount: number;        // in stroops (1 XLM = 10_000_000 stroops)
  fee: number;
  status: EscrowStatus;
  arbitrator: string | null;
  created_at: Date;
  updated_at: Date;
}

const STATUSES: EscrowStatus[] = ['created', 'funded', 'completed', 'disputed', 'cancelled'];

let _tradeIdSeq = 1;

export function escrowFactory(overrides: Partial<EscrowRecord> = {}): EscrowRecord {
  const amount = randInt(1_000_000, 100_000_000); // 0.1 – 10 XLM in stroops
  const fee = Math.floor(amount * 0.01);          // 1% fee
  const createdAt = faker.date.past({ years: 1 });
  return {
    trade_id: _tradeIdSeq++,
    seller: stellarAddress(),
    buyer: stellarAddress(),
    amount,
    fee,
    status: pick(STATUSES),
    arbitrator: faker.datatype.boolean() ? stellarAddress() : null,
    created_at: createdAt,
    updated_at: faker.date.between({ from: createdAt, to: new Date() }),
    ...overrides,
  };
}

/** Build N escrows, cycling through all statuses for coverage */
export function escrowList(n: number, overrides: Partial<EscrowRecord> = {}): EscrowRecord[] {
  return Array.from({ length: n }, (_, i) =>
    escrowFactory({ status: STATUSES[i % STATUSES.length], ...overrides })
  );
}

/** Reset the trade_id sequence (call between test suites) */
export function resetEscrowSeq(start = 1): void {
  _tradeIdSeq = start;
}
