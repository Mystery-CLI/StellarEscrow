<script lang="ts">
  import { themeStore, accentColors, type AccentColor, type FontSize } from '$lib/theme';

  let open = false;
  const accents = Object.keys(accentColors) as AccentColor[];
  const fontSizes: { value: FontSize; label: string }[] = [
    { value: 'sm', label: 'Small' },
    { value: 'md', label: 'Medium' },
    { value: 'lg', label: 'Large' },
  ];
</script>

<div class="relative">
  <!-- Toggle dark/light -->
  <button
    on:click={() => themeStore.toggle()}
    class="p-2 rounded-lg border border-[var(--accent)] text-[var(--accent)] hover:bg-[var(--accent)] hover:text-white transition-colors"
    title="Toggle theme"
    aria-label="Toggle dark/light mode"
  >
    {#if $themeStore.theme === 'dark'}☀️{:else}🌙{/if}
  </button>

  <!-- Settings gear -->
  <button
    on:click={() => (open = !open)}
    class="ml-1 p-2 rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
    title="Customize theme"
    aria-label="Open theme settings"
  >⚙️</button>

  {#if open}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div
    class="absolute right-0 mt-2 w-56 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl shadow-lg p-4 z-50"
    on:click|stopPropagation
  >
    <!-- Accent color -->
    <p class="text-xs font-semibold text-gray-500 dark:text-gray-400 mb-2 uppercase tracking-wide">Accent</p>
    <div class="flex gap-2 mb-4">
      {#each accents as accent}
        <button
          on:click={() => themeStore.setAccent(accent)}
          class="w-6 h-6 rounded-full border-2 transition-transform hover:scale-110"
          style="background:{accentColors[accent].bg}; border-color:{$themeStore.accent === accent ? '#000' : 'transparent'}"
          title={accent}
          aria-label="Set accent to {accent}"
        />
      {/each}
    </div>

    <!-- Font size -->
    <p class="text-xs font-semibold text-gray-500 dark:text-gray-400 mb-2 uppercase tracking-wide">Font Size</p>
    <div class="flex gap-1">
      {#each fontSizes as fs}
        <button
          on:click={() => themeStore.setFontSize(fs.value)}
          class="flex-1 py-1 text-xs rounded-lg border transition-colors
            {$themeStore.fontSize === fs.value
              ? 'bg-[var(--accent)] text-white border-[var(--accent)]'
              : 'border-gray-300 dark:border-gray-600 dark:text-gray-300 hover:border-[var(--accent)]'}"
        >{fs.label}</button>
      {/each}
    </div>
  </div>
  {/if}
</div>

<!-- Close panel on outside click -->
<svelte:window on:click={() => (open = false)} />
