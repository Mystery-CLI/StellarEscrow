<script lang="ts">
  import { onMount } from 'svelte';
  // Placeholder home page
</script>

<div class="container mx-auto px-6 py-12">
  <h1 class="text-4xl font-bold text-center mb-12">StellarEscrow</h1>
  <div class="max-w-2xl mx-auto">
    <p class="text-xl text-gray-600 mb-8 text-center">
      Decentralized escrow on Stellar/Soroban
    </p>
    <div class="grid md:grid-cols-2 gap-6">
      <a href="/trades/1" class="card p-6 hover:shadow-lg">
        <h3 class="text-2xl font-semibold mb-2">View Trade #1</h3>
        <p>Test funding interface</p>
      </a>
      <div class="card p-6">
        <h3 class="text-2xl font-semibold mb-2">Coming Soon</h3>
        <p>Create trades, buyer dashboard</p>
      </div>
    </div>
  </div>
</div>

<style>
  .card {
    @apply bg-white rounded-xl shadow-md border border-gray-200 transition-all;
  }
</style>
