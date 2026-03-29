/**
 * Load balancer — round-robin across upstream instances with the same pathPrefix.
 */

import { UpstreamConfig } from './types';

export class LoadBalancer {
  /** counter per upstream name-group */
  private counters = new Map<string, number>();

  /**
   * Select the upstream for a given request path.
   * Groups upstreams by pathPrefix and round-robins within the group.
   */
  select(path: string, upstreams: UpstreamConfig[]): UpstreamConfig | null {
    // Find all upstreams whose pathPrefix matches (longest prefix wins)
    const candidates = upstreams
      .filter((u) => path.startsWith(u.pathPrefix))
      .sort((a, b) => b.pathPrefix.length - a.pathPrefix.length);

    if (candidates.length === 0) return null;

    // Group by pathPrefix (longest match)
    const prefix = candidates[0].pathPrefix;
    const group = candidates.filter((u) => u.pathPrefix === prefix);

    if (group.length === 1) return group[0];

    // Round-robin within the group
    const key = prefix;
    const idx = (this.counters.get(key) ?? 0) % group.length;
    this.counters.set(key, idx + 1);
    return group[idx];
  }

  /** Reset counters (useful for testing) */
  reset(): void {
    this.counters.clear();
  }
}
