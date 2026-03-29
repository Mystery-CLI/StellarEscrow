import { writable } from 'svelte/store';

export const contractId = writable('CDVOID__...'); // Replace with deployed contract ID
export const network = writable<'testnet' | 'mainnet'>('testnet');
