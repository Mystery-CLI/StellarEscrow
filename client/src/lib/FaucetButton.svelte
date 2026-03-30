<script lang="ts">
  import { network } from '$lib/stores';

  // Faucet state
  let loading = false;
  let toast: { message: string; ok: boolean } | null = null;

  // Wallet address — read from localStorage (set by wallet connect flow)
  $: address = typeof localStorage !== 'undefined' ? localStorage.getItem('wallet_address') ?? '' : '';

  async function requestXlm() {
    if (!address) return { ok: false, message: 'No wallet connected' };
    const res = await fetch(`https://friendbot.stellar.org?addr=${encodeURIComponent(address)}`);
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(body?.detail ?? `Friendbot error ${res.status}`);
    }
    return { ok: true, message: 'XLM funded via Friendbot' };
  }

  async function requestUsdc() {
    if (!address) return { ok: false, message: 'No wallet connected' };
    // USDC testnet faucet — Circle's testnet faucet endpoint
    const res = await fetch('https://faucet.circle.com/api/faucet', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ address, blockchain: 'stellar', token: 'usdc' }),
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(body?.message ?? `USDC faucet error ${res.status}`);
    }
    return { ok: true, message: 'USDC funded via Circle testnet faucet' };
  }

  async function handleClick() {
    if (loading || $network !== 'testnet') return;
    loading = true;
    toast = null;

    try {
      // Run both requests; collect results independently so one failure doesn't block the other
      const [xlmResult, usdcResult] = await Promise.allSettled([requestXlm(), requestUsdc()]);

      const messages: string[] = [];
      let anyFailed = false;

      for (const r of [xlmResult, usdcResult]) {
        if (r.status === 'fulfilled') {
          messages.push(r.value.message);
        } else {
          messages.push((r.reason as Error).message);
          anyFailed = true;
        }
      }

      toast = { ok: !anyFailed, message: messages.join(' · ') };
    } catch (err) {
      toast = { ok: false, message: (err as Error).message };
    } finally {
      loading = false;
      // Auto-dismiss toast after 5 s
      setTimeout(() => { toast = null; }, 5000);
    }
  }
</script>

{#if $network === 'testnet'}
  <div class="relative inline-flex flex-col items-end gap-1">
    <button
      on:click={handleClick}
      disabled={loading || !address}
      aria-busy={loading}
      aria-label="Request testnet XLM and USDC tokens"
      class="inline-flex items-center gap-2 rounded-lg border border-indigo-400 bg-indigo-50 px-3 py-1.5 text-sm font-medium text-indigo-700
             hover:bg-indigo-100 disabled:cursor-not-allowed disabled:opacity-50
             dark:border-indigo-600 dark:bg-indigo-900/30 dark:text-indigo-300 dark:hover:bg-indigo-900/50
             transition-colors duration-150"
    >
      {#if loading}
        <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
        </svg>
        Requesting…
      {:else}
        <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path d="M10 2a8 8 0 100 16A8 8 0 0010 2zm1 11H9v-2h2v2zm0-4H9V7h2v2z" />
        </svg>
        Get Test Tokens
      {/if}
    </button>

    {#if toast}
      <div
        role="status"
        aria-live="polite"
        class="absolute top-full mt-1 right-0 z-50 max-w-xs rounded-lg px-3 py-2 text-xs shadow-lg
               {toast.ok
                 ? 'bg-green-50 text-green-800 border border-green-200 dark:bg-green-900/40 dark:text-green-300 dark:border-green-700'
                 : 'bg-red-50 text-red-800 border border-red-200 dark:bg-red-900/40 dark:text-red-300 dark:border-red-700'}"
      >
        {toast.message}
      </div>
    {/if}
  </div>
{/if}
