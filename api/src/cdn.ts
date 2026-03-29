import { ApiClient } from './client';

/**
 * CDN purge API — POST /api/cdn/purge
 * Accepts { paths: string[] } and issues cache-purge requests to the CDN provider.
 *
 * Set CDN_PURGE_URL and CDN_PURGE_TOKEN in environment variables.
 */
export async function handleCdnPurge(
  paths: string[],
  client: ApiClient
): Promise<{ purged: string[]; failed: string[] }> {
  const purgeUrl = process.env.CDN_PURGE_URL;
  const purgeToken = process.env.CDN_PURGE_TOKEN;

  if (!purgeUrl || !purgeToken) {
    // No CDN configured — treat as no-op success
    return { purged: paths, failed: [] };
  }

  const results = await Promise.allSettled(
    paths.map((path) =>
      fetch(purgeUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${purgeToken}`,
        },
        body: JSON.stringify({ files: [path] }),
      }).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return path;
      })
    )
  );

  const purged: string[] = [];
  const failed: string[] = [];

  results.forEach((result, i) => {
    if (result.status === 'fulfilled') purged.push(paths[i]);
    else failed.push(paths[i]);
  });

  return { purged, failed };
}
