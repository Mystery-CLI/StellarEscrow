/**
 * Pure TypeScript mirrors of the Rust compliance workflow logic.
 * Used in regression tests to verify the state machine hasn't changed.
 */

type ComplianceStatus = 'pending' | 'approved' | 'rejected' | 'requires_review' | 'blocked';

const ALLOWED_TRANSITIONS: [ComplianceStatus, ComplianceStatus][] = [
  ['pending', 'approved'],
  ['pending', 'rejected'],
  ['pending', 'requires_review'],
  ['pending', 'blocked'],
  ['requires_review', 'approved'],
  ['requires_review', 'rejected'],
  ['requires_review', 'blocked'],
];

export function is_valid_transition(from: string, to: string): boolean {
  return ALLOWED_TRANSITIONS.some(([f, t]) => f === from && t === to);
}
