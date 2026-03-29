/**
 * Synthetic Stellar address generator.
 * Produces structurally valid G-addresses (56 chars, base32 alphabet) using
 * only synthetic random data — no real keys or PII.
 */

const BASE32_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

export function stellarAddress(): string {
  // Stellar public keys: 'G' + 55 base32 chars = 56 chars total
  let addr = 'G';
  for (let i = 0; i < 55; i++) {
    addr += BASE32_CHARS[Math.floor(Math.random() * BASE32_CHARS.length)];
  }
  return addr;
}

/** Random integer in [min, max] inclusive */
export function randInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

/** Random element from an array */
export function pick<T>(arr: readonly T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}
