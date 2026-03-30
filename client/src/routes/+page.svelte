<script lang="ts">
  import { t } from '$lib/i18n';
</script>

<svelte:head>
  <title>{$t('nav.brand')}</title>
</svelte:head>

<div class="container mx-auto px-6 py-12">
  <h1 class="text-4xl font-bold text-center mb-12">{$t('home.title')}</h1>
  <div class="max-w-2xl mx-auto">
    <p class="text-xl text-gray-600 mb-8 text-center">
      {$t('home.subtitle')}
    </p>
    <div class="grid md:grid-cols-2 gap-6">
      <a href="/trades/1" class="card p-6 hover:shadow-lg">
        <h3 class="text-2xl font-semibold mb-2">{$t('home.viewTrade')}</h3>
        <p>{$t('home.viewTradeDesc')}</p>
      </a>
      <div class="card p-6">
        <h3 class="text-2xl font-semibold mb-2">{$t('home.comingSoon')}</h3>
        <p>{$t('home.comingSoonDesc')}</p>
      </div>
    </div>
  </div>
</div>

<style>
  .card {
    @apply bg-white rounded-xl shadow-md border border-gray-200 transition-all;
  }
</style>
