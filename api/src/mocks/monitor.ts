type CallRecord = { method: string; url: string; count: number };

const calls = new Map<string, CallRecord>();

export function recordCall(method: string, url: string) {
  const key = `${method} ${url}`;
  const existing = calls.get(key);
  if (existing) {
    existing.count++;
  } else {
    calls.set(key, { method, url, count: 1 });
  }
}

export function getCallCount(method: string, url: string): number {
  return calls.get(`${method} ${url}`)?.count ?? 0;
}

export function getCalls(): CallRecord[] {
  return Array.from(calls.values());
}

export function resetMonitor() {
  calls.clear();
}
