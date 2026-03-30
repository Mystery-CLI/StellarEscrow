<script lang="ts">
  import { isOnline, pushPermission, requestPushPermission } from '$lib/pwa';
  import { useRegisterSW } from 'virtual:pwa-register/svelte';

  const { needRefresh, updateServiceWorker } = useRegisterSW();

  const VAPID_KEY = import.meta.env.VITE_VAPID_PUBLIC_KEY ?? '';
</script>

<!-- Offline banner -->
{#if !$isOnline}
  <div role="alert" class="fixed bottom-0 inset-x-0 z-50 flex items-center justify-center gap-2 bg-gray-900 text-white text-sm py-2 px-4">
    <span>📡</span> You're offline — showing cached data
  </div>
{/if}

<!-- SW update prompt -->
{#if $needRefresh}
  <div role="alert" class="fixed bottom-4 right-4 z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl shadow-lg p-4 flex items-center gap-3 text-sm">
    <span>🔄 New version available</span>
    <button
      on:click={() => updateServiceWorker(true)}
      class="px-3 py-1 rounded-lg bg-[var(--accent)] text-white hover:opacity-90"
    >Update</button>
  </div>
{/if}

<!-- Push notification opt-in (shown once, only when granted is not yet set) -->
{#if $pushPermission === 'default' && $isOnline && VAPID_KEY}
  <div class="fixed bottom-4 left-4 z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl shadow-lg p-4 text-sm max-w-xs">
    <p class="font-semibold mb-1 dark:text-white">Stay updated</p>
    <p class="text-gray-500 dark:text-gray-400 mb-3">Get notified when your trades change status.</p>
    <div class="flex gap-2">
      <button
        on:click={requestPushPermission}
        class="flex-1 px-3 py-1.5 rounded-lg bg-[var(--accent)] text-white hover:opacity-90 text-xs font-medium"
      >Enable</button>
      <button
        on:click={() => pushPermission.set('denied')}
        class="px-3 py-1.5 rounded-lg border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 text-xs"
      >Not now</button>
    </div>
  </div>
{/if}
