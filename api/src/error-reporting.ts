/**
 * Centralised error reporting for the API client layer.
 *
 * - Structured in-memory log (last 100 entries)
 * - Remote reporting via POST /api/errors (best-effort, never throws)
 * - User-friendly message map for contract codes, HTTP statuses, named codes
 */

export interface ErrorEntry {
  ts: string;
  level: 'error' | 'warn';
  message: string;
  code?: string | number;
  status?: number;
  context?: Record<string, unknown>;
}

// ── In-memory log ─────────────────────────────────────────────────────────────

const _log: ErrorEntry[] = [];
const MAX_LOG = 100;

export function logError(
  message: string,
  opts: Omit<ErrorEntry, 'ts' | 'level' | 'message'> = {}
): ErrorEntry {
  const entry: ErrorEntry = { ts: new Date().toISOString(), level: 'error', message, ...opts };
  _log.unshift(entry);
  if (_log.length > MAX_LOG) _log.pop();
  console.error('[api-error]', message, opts);
  return entry;
}

export function logWarn(
  message: string,
  opts: Omit<ErrorEntry, 'ts' | 'level' | 'message'> = {}
): ErrorEntry {
  const entry: ErrorEntry = { ts: new Date().toISOString(), level: 'warn', message, ...opts };
  _log.unshift(entry);
  if (_log.length > MAX_LOG) _log.pop();
  console.warn('[api-warn]', message, opts);
  return entry;
}

export function getErrorLog(): ErrorEntry[] {
  return [..._log];
}

// ── Remote reporting ──────────────────────────────────────────────────────────

const REPORT_ENDPOINT = '/api/errors';

export async function reportError(
  error: unknown,
  context: Record<string, unknown> = {}
): Promise<void> {
  const message = error instanceof Error ? error.message : String(error);
  const stack = error instanceof Error ? error.stack : undefined;
  const entry = logError(message, { context: { stack, ...context } });
  try {
    await fetch(REPORT_ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(entry),
      keepalive: true, // survives page unload
    });
  } catch { /* best-effort — never throw */ }
}

// ── User-friendly message map ─────────────────────────────────────────────────

const MESSAGES: Record<string | number, string> = {
  // Contract error codes
  1:  'This contract has already been set up.',
  2:  'Contract is not yet initialized.',
  3:  'Amount must be greater than zero.',
  4:  'Fee must be between 0 and 100%.',
  5:  'This arbitrator is not registered.',
  6:  'Trade not found. Check the trade ID.',
  7:  'This action is not allowed in the current trade state.',
  8:  'A calculation error occurred. Please try again.',
  9:  'No fees available to withdraw.',
  10: 'You are not authorized to perform this action.',
  11: 'The contract is currently paused.',
  // HTTP status codes
  400: 'Invalid request. Please check your input.',
  401: 'Session expired. Please reconnect your wallet.',
  403: 'You do not have permission to do that.',
  404: 'The requested resource was not found.',
  408: 'Request timed out. Please try again.',
  429: 'Too many requests. Please wait a moment.',
  500: 'Server error. Our team has been notified.',
  503: 'Service unavailable. You may be offline.',
  // Named API codes
  DATABASE_ERROR: 'A database error occurred. Please try again shortly.',
  STELLAR_ERROR:  'Could not reach the Stellar network. Check your connection.',
  NETWORK_ERROR:  'Network error. Please check your internet connection.',
  INVALID_FORMAT: 'Unexpected data format received.',
  RATE_LIMITED:   'Too many requests. Please slow down.',
};

export function friendlyMessage(error: unknown): string {
  if (!error) return 'An unexpected error occurred. Please try again.';
  const e = error as any;
  if (e.code !== undefined) return MESSAGES[e.code] ?? e.message ?? 'An unexpected error occurred.';
  if (e.status !== undefined) return MESSAGES[e.status] ?? 'An unexpected error occurred.';
  if (error instanceof TypeError && String(error.message).includes('fetch')) return MESSAGES[503];
  return (error instanceof Error ? error.message : String(error)) || 'An unexpected error occurred.';
}
