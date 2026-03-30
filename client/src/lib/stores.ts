import { writable } from 'svelte/store';
import { env } from '$lib/env';

export const contractId = writable('CDVOID__...'); // Replace with deployed contract ID
export const network = writable<'testnet' | 'mainnet'>(env.network);
