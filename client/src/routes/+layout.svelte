<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { themeStore } from '$lib/theme';
  import ThemeToggle from '$lib/ThemeToggle.svelte';
  import SearchBar from '$lib/SearchBar.svelte';
  import OfflineIndicator from '$lib/OfflineIndicator.svelte';
  import LanguageSwitcher from '$lib/LanguageSwitcher.svelte';
  import { collectWebVitals } from '$lib/perf';
  import { setupI18n, textDir, t } from '$lib/i18n';
  import { isLoading } from 'svelte-i18n';

  // Initialise i18n before mount so translations are available immediately
  setupI18n();

  onMount(() => {
    themeStore.init();
    collectWebVitals((m) => {
      // In production, forward to your analytics endpoint here
      if (import.meta.env.DEV) console.info(`[vitals] ${m.name}: ${m.value.toFixed(1)} (${m.rating})`);
    });
  });
</script>

<svelte:head>
  <!-- dir is set imperatively in setLocale(); this keeps SSR/initial render correct -->
</svelte:head>

<div
  class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors duration-200"
  dir={$textDir}
  lang="auto"
>
  <header class="border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-6 py-3 flex items-center gap-4">
    <a href="/" class="font-bold text-lg text-[var(--accent)] shrink-0">{$t('nav.brand')}</a>
    <div class="flex-1"><SearchBar /></div>
    <LanguageSwitcher />
    <ThemeToggle />
  </header>
  <main>
    {#if $isLoading}
      <!-- Thin loading bar while locale bundle fetches -->
      <div class="h-0.5 w-full bg-[var(--accent)] animate-pulse"></div>
    {:else}
      <slot />
    {/if}
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

  /* RTL layout support */
  [dir='rtl'] {
    font-family: 'Segoe UI', 'Tahoma', 'Arial', sans-serif;
  }

  [dir='rtl'] .ms-auto { margin-inline-start: auto; }
  [dir='rtl'] .me-auto { margin-inline-end: auto; }
</style>
