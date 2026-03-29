<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { FundingPreview, escrowClient } from '$lib/contract';

  let tradeId = $page.params.id;
  let loading = true;
  let preview: FundingPreview | null = null;
  let modalOpen = false;
  let approving = false;
  let funding = false;
  let txHash = '';
  let success = false;
  let error = '';

  let isBuyer = false; // Simulate - from wallet connect
  let tradeStatus = 'Created'; // Simulate from get_trade_detail

  async function loadPreview() {
    try {
      // preview = await escrowClient.getFundingPreview(parseInt(tradeId), 'buyer_address', 'source');
      loading = false;
      // Mock data for now
      preview = {
        trade_id: BigInt(tradeId),
        buyer: 'GBUYER...',
        seller: 'GSELLER...',
        amount: 10000000n, // 10 USDC
        fee: 50000n, // 0.05 USDC
        buyer_balance: 50000000n, // 50 USDC
        allowance_sufficient: false
      };
    } catch (e) {
      error = 'Failed to load preview';
      loading = false;
    }
  }

  async function handleApprove() {
    approving = true;
    try {
      // await approveUSDC(escrow.contractId, preview.amount);
      approving = false;
      preview!.allowance_sufficient = true;
    } catch (e) {
      error = 'Approval failed';
      approving = false;
    }
  }

  async function handleFund() {
    funding = true;
    try {
      // txHash = await escrowClient.fundTrade(parseInt(tradeId), 'buyerKeypair', preview!);
      txHash = 'mock_tx_123';
      success = true;
      tradeStatus = 'Funded';
      modalOpen = false;
    } catch (e) {
      error = 'Funding failed';
    }
    funding = false;
  }

  $: if (preview && preview.allowance_sufficient) {
    // Ready to fund
  }
</script>

<svelte:head>
  <title>Trade #{tradeId} - StellarEscrow</title>
</svelte:head>

<div class="container mx-auto px-6 py-12">
  <a href="/" class="mb-8 inline-flex items-center text-blue-600 hover:text-blue-800">
    ← Back to trades
  </a>

  {#if loading}
    <div class="text-center py-12">Loading trade details...</div>
  {:else}
    <div class="max-w-4xl mx-auto">
      <!-- Trade Header -->
      <div class="bg-white rounded-2xl shadow-xl p-8 mb-8">
        <div class="flex justify-between items-start mb-6">
          <div>
            <h1 class="text-3xl font-bold">Trade #{tradeId}</h1>
            <div class="flex gap-4 mt-2 text-sm text-gray-500">
              <span class="px-3 py-1 bg-blue-100 rounded-full">Status: {tradeStatus}</span>
              <span>Created: 2 hours ago</span>
            </div>
          </div>
          {#if isBuyer && tradeStatus === 'Created'}
            <button 
              on:click={() => { modalOpen = true; loadPreview(); }}
              class="bg-gradient-to-r from-blue-500 to-indigo-600 text-white px-8 py-3 rounded-xl font-semibold hover:shadow-lg transition-all"
            >
              Fund Trade
            </button>
          {/if}
        </div>

        <div class="grid md:grid-cols-2 gap-8">
          <div>
            <h3 class="font-semibold mb-3">Seller</h3>
            <p class="text-sm text-gray-600">GSELLER123...</p>
          </div>
          <div>
            <h3 class="font-semibold mb-3">Buyer</h3>
            <p class="text-sm text-gray-600 bg-yellow-50 px-3 py-1 rounded">GBUYER456... (you)</p>
          </div>
          <div>
            <h3 class="font-semibold mb-3">Amount</h3>
            <p class="text-2xl font-bold text-gray-900">${preview ? Number(preview.amount / 1000000n) : 0}.00 USDC</p>
          </div>
          <div>
            <h3 class="font-semibold mb-3">Platform Fee</h3>
            <p class="text-xl font-semibold text-red-600">${preview ? Number(preview.fee / 1000000n) : 0}.00 USDC</p>
          </div>
        </div>
      </div>

      <!-- Timeline -->
      <div class="bg-white rounded-2xl shadow-xl p-8">
        <h2 class="text-2xl font-bold mb-6">Status Timeline</h2>
        <div class="space-y-4">
          <div class="flex items-center">
            <div class="w-6 h-6 bg-green-500 rounded-full"></div>
            <div class="ml-4 flex-1">
              <p class="font-semibold">Trade Created</p>
              <p class="text-sm text-gray-500">2 hours ago</p>
            </div>
          </div>
          {#if tradeStatus === 'Funded'}
            <div class="flex items-center">
              <div class="w-6 h-6 bg-blue-500 rounded-full"></div>
              <div class="ml-4 flex-1">
                <p class="font-semibold">Trade Funded</p>
                <p class="text-sm text-gray-500">{txHash || 'Tx Hash'}</p>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Funding Modal -->
{#if modalOpen}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-3xl p-8 max-w-md w-full max-h-[90vh] overflow-y-auto">
      <div class="flex justify-between items-center mb-6">
        <h2 class="text-2xl font-bold">Fund Trade #{tradeId}</h2>
        <button on:click={() => modalOpen = false} class="text-gray-500 hover:text-gray-700">
          ✕
        </button>
      </div>

      {#if !preview}
        <div class="text-center py-8">Loading preview...</div>
      {:else}
        <div class="space-y-6">
          <!-- Summary -->
          <div class="bg-gray-50 p-6 rounded-2xl">
            <h3 class="font-semibold mb-4">You'll pay:</h3>
            <div class="space-y-2">
              <div class="flex justify-between">
                <span>Amount:</span>
                <span class="font-semibold">${Number(preview.amount / 1000000n)}.00 USDC</span>
              </div>
              <div class="flex justify-between pt-2 border-t">
                <span>Platform fee:</span>
                <span class="text-red-600 font-semibold">${Number(preview.fee / 1000000n)}.00 USDC</span>
              </div>
            </div>
          </div>

          <!-- Balance & Allowance -->
          <div class="space-y-3">
            <div class="flex justify-between">
              <span>Balance:</span>
              <span class="font-semibold">${Number(preview.buyer_balance / 1000000n)}.00 USDC ✓</span>
            </div>
            {#if !preview.allowance_sufficient}
              <div class="bg-yellow-50 border border-yellow-200 rounded-xl p-4">
                <p class="text-yellow-800 text-sm">
                  <strong>Allowance needed:</strong> ${Number(preview.amount / 1000000n)}.00 USDC to escrow contract
                </p>
                <button 
                  on:click={handleApprove}
                  disabled={approving}
                  class="mt-3 w-full bg-yellow-500 hover:bg-yellow-600 text-white py-2 px-4 rounded-xl font-medium transition-colors {approving ? 'opacity-50 cursor-not-allowed' : ''}"
                >
                  {#if approving} Approving... {:else} Approve USDC {/if}
                </button>
              </div>
            {:else}
              <div class="flex justify-between text-green-600 font-semibold">
                <span>USDC Approved ✓</span>
                <span class="text-sm">Ready to fund</span>
              </div>
            {/if}
          </div>

          <!-- Actions -->
          <div class="flex gap-3 pt-4">
            <button 
              on:click={() => modalOpen = false}
              class="flex-1 bg-gray-200 hover:bg-gray-300 text-gray-800 py-3 px-6 rounded-xl font-medium transition-colors"
            >
              Cancel
            </button>
            {#if preview.allowance_sufficient}
              <button 
                on:click={handleFund}
                disabled={funding}
                class="flex-1 bg-gradient-to-r from-green-500 to-emerald-600 hover:from-green-600 hover:to-emerald-700 text-white py-3 px-6 rounded-xl font-semibold transition-all shadow-lg hover:shadow-xl {funding ? 'opacity-50 cursor-not-allowed' : ''}"
              >
                {#if funding}
                  <span>Funding... </span><span class="animate-spin">⏳</span>
                {:else}
                  Confirm & Fund
                {/if}
              </button>
            {/if}
          </div>

          {#if error}
            <div class="bg-red-50 border border-red-200 rounded-xl p-4 mt-4">
              <p class="text-red-800 text-sm">{error}</p>
            </div>
          {/if}

          {#if success}
            <div class="bg-green-50 border border-green-200 rounded-xl p-4 mt-4">
              <p class="text-green-800 font-semibold">Trade funded successfully!</p>
              <p class="text-sm text-green-700 mt-1">Tx: {txHash}</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  /* Custom animations */
</style>
