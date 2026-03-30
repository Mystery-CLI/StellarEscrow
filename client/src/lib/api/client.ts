import { ApiRequestError } from './types';

// ---------------------------------------------------------------------------
// Simple in-memory GET cache with TTL
// ---------------------------------------------------------------------------
interface CacheEntry { value: unknown; expiresAt: number }
const cache = new Map<string, CacheEntry>();

export function cachedFetch<T>(
  key: string,
  ttlMs: number,
  fetcher: () => Promise<T>
): Promise<T> {
  const hit = cache.get(key);
  if (hit && Date.now() < hit.expiresAt) return Promise.resolve(hit.value as T);
  return fetcher().then((value) => {
    cache.set(key, { value, expiresAt: Date.now() + ttlMs });
    return value;
  });
}

export function invalidateCache(prefix?: string) {
  if (!prefix) { cache.clear(); return; }
  for (const key of cache.keys()) if (key.startsWith(prefix)) cache.delete(key);
}

const RETRY_STATUSES = new Set([429, 502, 503, 504]);
const DEFAULT_RETRIES = 3;
const BASE_DELAY_MS = 300;

export interface ClientConfig {
  baseUrl: string;
  retries?: number;
  /** Called before every request — return modified init or throw to abort */
  onRequest?: (url: string, init: RequestInit) => RequestInit | Promise<RequestInit>;
  /** Called after every successful response */
  onResponse?: (url: string, res: Response) => void;
}

async function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

export async function apiFetch<T>(
  config: ClientConfig,
  path: string,
  init: RequestInit = {}
): Promise<T> {
  const url = `${config.baseUrl}${path}`;
  const maxAttempts = (config.retries ?? DEFAULT_RETRIES) + 1;

  let attempt = 0;
  while (attempt < maxAttempts) {
    let reqInit: RequestInit = {
      headers: { 'Content-Type': 'application/json', ...init.headers },
      ...init,
    };

    if (config.onRequest) {
      reqInit = await config.onRequest(url, reqInit);
    }

    let res: Response;
    try {
      res = await fetch(url, reqInit);
    } catch (err) {
      if (attempt < maxAttempts - 1) {
        await sleep(BASE_DELAY_MS * 2 ** attempt);
        attempt++;
        continue;
      }
      throw new ApiRequestError(0, 'NETWORK_ERROR', (err as Error).message);
    }

    if (res.ok) {
      config.onResponse?.(url, res);
      return res.json() as Promise<T>;
    }

    if (RETRY_STATUSES.has(res.status) && attempt < maxAttempts - 1) {
      const retryAfter = res.headers.get('Retry-After');
      const delay = retryAfter ? parseInt(retryAfter) * 1000 : BASE_DELAY_MS * 2 ** attempt;
      await sleep(delay);
      attempt++;
      continue;
    }

    let code = 'API_ERROR';
    let message = res.statusText;
    try {
      const body = await res.json();
      code = body.code ?? code;
      message = body.message ?? message;
    } catch {
      // non-JSON error body
    }
    throw new ApiRequestError(res.status, code, message);
  }

  throw new ApiRequestError(0, 'MAX_RETRIES', 'Max retries exceeded');
}
