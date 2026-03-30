<script lang="ts">
  import { goto } from '$app/navigation';
  import { searchQuery, suggestions, searchHistory, fetchSuggestions, runSearch } from '$lib/search';

  let focused = false;
  let inputEl: HTMLInputElement;

  function onInput() {
    fetchSuggestions($searchQuery);
  }

  async function submit() {
    if (!$searchQuery.trim()) return;
    await goto(`/search?q=${encodeURIComponent($searchQuery)}`);
    focused = false;
    inputEl?.blur();
  }

  function pick(value: string, type: string) {
    if (type === 'trade') { goto(`/trades/${value}`); }
    else { searchQuery.set(value); submit(); }
    focused = false;
  }

  function pickHistory(q: string) {
    searchQuery.set(q);
    submit();
  }

  $: showDropdown = focused && ($suggestions.length > 0 || ($searchQuery === '' && $searchHistory.length > 0));
</script>

<div class="relative w-full max-w-sm">
  <form on:submit|preventDefault={submit} class="flex items-center">
    <input
      bind:this={inputEl}
      bind:value={$searchQuery}
      on:input={onInput}
      on:focus={() => (focused = true)}
      type="search"
      placeholder="Search trades, users, arbitrators…"
      class="w-full px-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600
             bg-white dark:bg-gray-700 dark:text-gray-100
             focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
    />
  </form>

  {#if showDropdown}
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="absolute top-full mt-1 w-full bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl shadow-lg z-50 overflow-hidden">
    {#if $searchQuery === '' && $searchHistory.length > 0}
      <p class="px-3 pt-2 pb-1 text-xs text-gray-400 uppercase tracking-wide">Recent</p>
      {#each $searchHistory as h}
        <button
          on:click={() => pickHistory(h)}
          class="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-200 flex items-center gap-2"
        >🕐 {h}</button>
      {/each}
    {:else}
      {#each $suggestions as s}
        <button
          on:click={() => pick(s.value, s.type)}
          class="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-200 flex items-center gap-2"
        >
          <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400">{s.type}</span>
          {s.label}
        </button>
      {/each}
    {/if}
  </div>
  {/if}
</div>

<svelte:window on:click={() => (focused = false)} />
