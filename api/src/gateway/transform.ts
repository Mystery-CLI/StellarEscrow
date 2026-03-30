/**
 * Request/response transformation middleware.
 *
 * CLI → Gateway: normalises CLI-friendly JSON to the format expected by
 * the Stellar indexer / Soroban RPC.
 *
 * Gateway → CLI: wraps upstream responses in a consistent envelope.
 */

/** Normalise a CLI trade-creation payload to the indexer's expected shape. */
export function transformRequest(path: string, body: unknown): unknown {
  if (!body || typeof body !== 'object') return body;
  const b = body as Record<string, unknown>;

  // CLI sends camelCase; indexer expects snake_case for trade creation
  if (path.startsWith('/events/replay') || path.startsWith('/trades')) {
    return toSnakeCase(b);
  }

  return body;
}

/** Wrap an upstream response in the standard gateway envelope. */
export function transformResponse(
  upstreamBody: unknown,
  upstreamStatus: number,
  upstream: string,
  latencyMs: number
): unknown {
  // Pass-through errors as-is
  if (upstreamStatus >= 400) return upstreamBody;

  return {
    data: upstreamBody,
    meta: {
      upstream,
      latencyMs,
      timestamp: new Date().toISOString(),
    },
  };
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function toSnakeCase(obj: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    const snakeKey = key.replace(/([A-Z])/g, '_$1').toLowerCase();
    result[snakeKey] =
      value && typeof value === 'object' && !Array.isArray(value)
        ? toSnakeCase(value as Record<string, unknown>)
        : value;
  }
  return result;
}
