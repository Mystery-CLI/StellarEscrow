<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { themeStore } from '$lib/theme';
  import ThemeToggle from '$lib/ThemeToggle.svelte';
  import SearchBar from '$lib/SearchBar.svelte';
  import OfflineIndicator from '$lib/OfflineIndicator.svelte';
  import FaucetButton from '$lib/FaucetButton.svelte';
  import { collectWebVitals } from '$lib/perf';

  onMount(() => {
    themeStore.init();
    collectWebVitals((m) => {
      // In production, forward to your analytics endpoint here
      if (import.meta.env.DEV) console.info(`[vitals] ${m.name}: ${m.value.toFixed(1)} (${m.rating})`);
    });
  });
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors duration-200">
  <header class="border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-6 py-3 flex items-center gap-4">
    <a href="/" class="font-bold text-lg text-[var(--accent)] shrink-0">StellarEscrow</a>
    <div class="flex-1"><SearchBar /></div>
    <FaucetButton />
    <ThemeToggle />
  </header>
  <main>
    <slot />
  </main>
  <OfflineIndicator />
</div>

<style global lang="postcss">
  @tailwind base;
  @tailwind components;
  @tailwind utilities;

  :root {
    --accent: #6366f1;
    --font-size-base: 16px;
  }

  html {
    font-size: var(--font-size-base);
  }
</style>
