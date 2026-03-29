/**
 * StellarEscrow Error Handling Module
 * - Global error boundary (unhandledrejection + onerror)
 * - Structured error logging with remote reporting
 * - User-friendly messages mapped from error codes
 * - Retry with exponential backoff
 * - Error UI (modal + inline)
 */

// ── Error code → user message map ────────────────────────────────────────────

const ERROR_MESSAGES = {
  // Contract errors (codes 1–26)
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
  429: 'Too many requests. Please wait a moment and try again.',
  500: 'Server error. Our team has been notified.',
  503: 'Service unavailable. You may be offline.',
  // Named codes from API
  DATABASE_ERROR:    'A database error occurred. Please try again shortly.',
  STELLAR_ERROR:     'Could not reach the Stellar network. Check your connection.',
  NETWORK_ERROR:     'Network error. Please check your internet connection.',
  INVALID_FORMAT:    'Unexpected data format received.',
  EVENT_NOT_FOUND:   'Event not found.',
  RATE_LIMITED:      'Too many requests. Please slow down.',
  // Fallback
  DEFAULT: 'Something went wrong. Please try again.',
};

export function friendlyMessage(error) {
  if (!error) return ERROR_MESSAGES.DEFAULT;
  // Numeric contract/HTTP code
  if (typeof error.code === 'number') return ERROR_MESSAGES[error.code] ?? ERROR_MESSAGES.DEFAULT;
  // String API code
  if (typeof error.code === 'string') return ERROR_MESSAGES[error.code] ?? ERROR_MESSAGES.DEFAULT;
  // HTTP Response status
  if (error instanceof Response || error.status) return ERROR_MESSAGES[error.status] ?? ERROR_MESSAGES.DEFAULT;
  // Network failure
  if (error instanceof TypeError && error.message.includes('fetch')) return ERROR_MESSAGES[503];
  return ERROR_MESSAGES.DEFAULT;
}

// ── Error logger ──────────────────────────────────────────────────────────────

const _log = [];
const MAX_LOG = 100;

function log(level, message, context = {}) {
  const entry = { level, message, context, ts: new Date().toISOString() };
  _log.unshift(entry);
  if (_log.length > MAX_LOG) _log.pop();
  console[level === 'error' ? 'error' : 'warn'](`[${level}]`, message, context);
  return entry;
}

export function logError(message, context = {}) { return log('error', message, context); }
export function logWarn(message, context = {})  { return log('warn',  message, context); }
export function getErrorLog() { return [..._log]; }

// ── Remote error reporting ────────────────────────────────────────────────────

export function reportError(error, context = {}) {
  const entry = logError(error?.message ?? String(error), {
    stack: error?.stack,
    ...context,
  });
  // Best-effort beacon; never throws
  try {
    navigator.sendBeacon?.('/api/errors', JSON.stringify(entry));
  } catch { /* ignore */ }
}

// ── Global error boundary ─────────────────────────────────────────────────────

export function initErrorBoundary() {
  window.addEventListener('error', (event) => {
    reportError(event.error ?? new Error(event.message), {
      source: event.filename,
      line: event.lineno,
      col: event.colno,
    });
    showErrorUI(friendlyMessage(event.error), { fatal: false });
  });

  window.addEventListener('unhandledrejection', (event) => {
    const err = event.reason instanceof Error ? event.reason : new Error(String(event.reason));
    reportError(err, { type: 'unhandledrejection' });
    showErrorUI(friendlyMessage(event.reason), { fatal: false });
    event.preventDefault(); // suppress console noise after we've handled it
  });
}

// ── Retry with exponential backoff ────────────────────────────────────────────

/**
 * Retry an async function up to `maxAttempts` times with exponential backoff.
 * @param {() => Promise<T>} fn
 * @param {{ maxAttempts?: number, baseDelay?: number, onRetry?: (attempt, err) => void }} [opts]
 * @returns {Promise<T>}
 */
export async function withRetry(fn, { maxAttempts = 3, baseDelay = 500, onRetry } = {}) {
  let lastErr;
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (err) {
      lastErr = err;
      if (attempt === maxAttempts) break;
      const delay = baseDelay * 2 ** (attempt - 1);
      onRetry?.(attempt, err);
      logWarn(`Retry ${attempt}/${maxAttempts} after ${delay}ms`, { error: err?.message });
      await new Promise((r) => setTimeout(r, delay));
    }
  }
  throw lastErr;
}

// ── Error UI ──────────────────────────────────────────────────────────────────

/**
 * Show an error message to the user.
 * Uses the #error-modal if present, otherwise falls back to a toast.
 * @param {string} message
 * @param {{ fatal?: boolean, retryFn?: () => void }} [opts]
 */
export function showErrorUI(message, { fatal = false, retryFn } = {}) {
  const modal = document.getElementById('error-modal');
  if (modal) {
    document.getElementById('error-modal-message').textContent = message;
    const retryBtn = document.getElementById('error-modal-retry');
    if (retryBtn) {
      retryBtn.hidden = !retryFn;
      retryBtn.onclick = () => { hideErrorModal(); retryFn?.(); };
    }
    modal.hidden = false;
    modal.setAttribute('aria-hidden', 'false');
    document.getElementById('error-modal-close')?.focus();
    return;
  }
  // Fallback: inline toast (works even before DOM is fully ready)
  const container = document.getElementById('toast-container');
  if (!container) return;
  const toast = document.createElement('div');
  toast.className = `toast error${fatal ? ' fatal' : ''}`;
  toast.setAttribute('role', 'alert');
  toast.textContent = message;
  if (retryFn) {
    const btn = document.createElement('button');
    btn.textContent = 'Retry';
    btn.className = 'btn btn-sm';
    btn.onclick = retryFn;
    toast.appendChild(btn);
  }
  container.appendChild(toast);
  if (!fatal) setTimeout(() => { toast.style.opacity = '0'; setTimeout(() => toast.remove(), 300); }, 7000);
}

export function hideErrorModal() {
  const modal = document.getElementById('error-modal');
  if (!modal) return;
  modal.hidden = true;
  modal.setAttribute('aria-hidden', 'true');
}
