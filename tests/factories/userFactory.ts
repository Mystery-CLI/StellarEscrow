import { faker } from '@faker-js/faker';
import { stellarAddress } from './stellar';

export interface UserRecord {
  address: string;       // Stellar G-address (synthetic)
  display_name: string;  // Faker-generated, no real PII
  kyc_verified: boolean;
  created_at: Date;
}

export function userFactory(overrides: Partial<UserRecord> = {}): UserRecord {
  return {
    address: stellarAddress(),
    display_name: faker.internet.username(),   // synthetic username, no real name
    kyc_verified: faker.datatype.boolean(),
    created_at: faker.date.past({ years: 1 }),
    ...overrides,
  };
}

/** Build N users */
export function userList(n: number, overrides: Partial<UserRecord> = {}): UserRecord[] {
  return Array.from({ length: n }, () => userFactory(overrides));
}
