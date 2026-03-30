<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import {
    searchQuery, statusFilter, minAmount, maxAmount,
    trades, users, loading, error, searchHistory,
    runSearch, clearHistory
  } from '$lib/search';

  const STATUS_OPTIONS = ['', 'created', 'funded', 'completed', 'disputed'];

  onMount(() => {
    const q = $page.url.searchParams.get('q') ?? '';
    searchQuery.set(q);
    if (q) runSearch();
  });

  const fmt = (n: number) => `$${(n / 1_000_000).toFixed(2)}`;

  const statusColor: Record<string, string> = {
    created: 'bg-blue-100 text-blue-700',
    funded: 'bg-yellow-100 text-yellow-700',
    completed: 'bg-green-100 text-green-700',
    disputed: 'bg-red-100 text-red-700',
  };

  const roleColor: Record<string, string> = {
    buyer: 'bg-blue-100 text-blue-700',
    seller: 'bg-purple-100 text-purple-700',
    arbitrator: 'bg-orange-100 text-orange-700',
  };
</script>

<svelte:head><title>Search — StellarEscrow</title></svelte:head>

<div class="container mx-auto px-6 py-10 max-w-4xl">
  <h1 class="text-2xl font-bold mb-6 dark:text-white">Search</h1>

  <!-- Search form -->
  <form on:submit|preventDefault={runSearch} class="flex gap-2 mb-6">
    <input
      bind:value={$searchQuery}
      type="search"
      placeholder="Trade ID, address, status…"
      class="flex-1 px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
    />
    <button
      type="submit"
      class="px-5 py-2 rounded-lg bg-[var(--accent)] text-white font-medium hover:opacity-90 transition-opacity"
    >Search</button>
  </form>

  <!-- Filters -->
  <details class="mb-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl p-4">
    <summary class="cursor-pointer text-sm font-semibold text-gray-600 dark:text-gray-300">Filters</summary>
    <div class="mt-4 grid sm:grid-cols-3 gap-4">
      <div>
        <label class="block text-xs text-gray-500 mb-1">Status</label>
        <select bind:value={$statusFilter} class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 dark:text-white">
          {#each STATUS_OPTIONS as s}
            <option value={s}>{s || 'Any'}</option>
          {/each}
        </select>
      </div>
      <div>
        <label class="block text-xs text-gray-500 mb-1">Min Amount (USDC)</label>
        <input type="number" bind:value={$minAmount} min="0" placeholder="0"
          class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 dark:text-white" />
      </div>
      <div>
        <label class="block text-xs text-gray-500 mb-1">Max Amount (USDC)</label>
        <input type="number" bind:value={$maxAmount} min="0" placeholder="∞"
          class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 dark:text-white" />
      </div>
    </div>
  </details>

  {#if $error}
    <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-700 rounded-xl text-red-700 dark:text-red-300 text-sm">{$error}</div>
  {/if}

  {#if $loading}
    <div class="text-center py-16 text-gray-400">Searching…</div>
  {:else if $trades.length === 0 && $users.length === 0 && $searchQuery}
    <div class="text-center py-16 text-gray-400">No results for "{$searchQuery}"</div>
  {:else}
    <!-- Trades -->
    {#if $trades.length > 0}
      <section class="mb-8">
        <h2 class="text-lg font-semibold mb-3 dark:text-white">Trades <span class="text-sm font-normal text-gray-400">({$trades.length})</span></h2>
        <div class="space-y-2">
          {#each $trades as t}
            <a href="/trades/{t.trade_id}"
              class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl hover:border-[var(--accent)] transition-colors">
              <div>
                <span class="font-semibold dark:text-white">Trade #{t.trade_id}</span>
                <p class="text-xs text-gray-500 mt-0.5">{t.seller.slice(0,12)}… → {t.buyer.slice(0,12)}…</p>
              </div>
              <div class="text-right">
                <p class="font-semibold dark:text-white">{fmt(t.amount)}</p>
                <span class="text-xs px-2 py-0.5 rounded-full {statusColor[t.status] ?? 'bg-gray-100 text-gray-600'}">{t.status}</span>
              </div>
            </a>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Users / Arbitrators -->
    {#if $users.length > 0}
      <section class="mb-8">
        <h2 class="text-lg font-semibold mb-3 dark:text-white">Users & Arbitrators <span class="text-sm font-normal text-gray-400">({$users.length})</span></h2>
        <div class="space-y-2">
          {#each $users as u}
            <div class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl">
              <div>
                <p class="font-mono text-sm dark:text-white">{u.address}</p>
                <span class="text-xs px-2 py-0.5 rounded-full {roleColor[u.role] ?? 'bg-gray-100 text-gray-600'}">{u.role}</span>
              </div>
              <p class="text-sm text-gray-500">{u.trade_count} trades</p>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}

  <!-- Search History -->
  {#if $searchHistory.length > 0}
    <section>
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">Recent Searches</h2>
        <button on:click={clearHistory} class="text-xs text-gray-400 hover:text-red-500 transition-colors">Clear</button>
      </div>
      <div class="flex flex-wrap gap-2">
        {#each $searchHistory as h}
          <button
            on:click={() => { searchQuery.set(h); runSearch(); }}
            class="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 dark:text-gray-300 rounded-full hover:bg-[var(--accent)] hover:text-white transition-colors"
          >{h}</button>
        {/each}
      </div>
    </section>
  {/if}
</div>
