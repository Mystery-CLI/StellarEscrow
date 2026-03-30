// @ts-nocheck
import type { PageLoad } from './$types';
import { indexerApi } from '$lib/api';

// Preload trade events alongside the route chunk so the page renders with data immediately
export const load = async ({ params }: Parameters<PageLoad>[0]) => {
  const tradeId = parseInt(params.id);
  const events = await indexerApi.getTradeEvents(tradeId).catch(() => []);
  return { tradeId, events };
};
