<script lang="ts">
  import { locale } from 'svelte-i18n';
  import { setLocale, LOCALE_META, t } from '$lib/i18n';

  let open = false;

  $: currentMeta = LOCALE_META.find((m) => m.code === $locale) ?? LOCALE_META[0];

  function select(code) {
    setLocale(code);
    open = false;
  }

  function handleKeydown(e) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="relative" role="region" aria-label={$t('language.select')}>
  <button
    type="button"
    class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium
           bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600
           border border-gray-200 dark:border-gray-600
           text-gray-700 dark:text-gray-200 transition-colors focus:outline-none
           focus:ring-2 focus:ring-[var(--accent)] focus:ring-offset-1"
    aria-haspopup="listbox"
    aria-expanded={open}
    on:click={() => (open = !open)}
  >
    <!-- Globe icon -->
    <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
    </svg>
    <span>{currentMeta.nativeLabel}</span>
    <svg class="w-3 h-3 shrink-0 transition-transform {open ? 'rotate-180' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
      <polyline points="6 9 12 15 18 9"/>
    </svg>
  </button>

  {#if open}
    <!-- Backdrop -->
    <button
      class="fixed inset-0 z-40"
      aria-hidden="true"
      tabindex="-1"
      on:click={() => (open = false)}
    ></button>

    <!-- Dropdown -->
    <ul
      role="listbox"
      aria-label={$t('language.select')}
      class="absolute end-0 z-50 mt-1 w-44 rounded-xl shadow-lg
             bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
             py-1 overflow-hidden"
    >
      {#each LOCALE_META as meta}
        <li role="option" aria-selected={$locale === meta.code}>
          <button
            type="button"
            class="w-full text-start px-4 py-2 text-sm flex items-center justify-between gap-2
                   hover:bg-gray-50 dark:hover:bg-gray-700
                   {$locale === meta.code ? 'font-semibold text-[var(--accent)]' : 'text-gray-700 dark:text-gray-200'}"
            dir={meta.dir}
            on:click={() => select(meta.code)}
          >
            <span>{meta.nativeLabel}</span>
            {#if meta.dir === 'rtl'}
              <span class="text-xs text-gray-400 font-normal">RTL</span>
            {/if}
            {#if $locale === meta.code}
              <svg class="w-4 h-4 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
