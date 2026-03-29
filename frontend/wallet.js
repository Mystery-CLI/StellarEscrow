/**
 * Stellar Wallet Integration Module
 * Supports: Freighter, Albedo, WalletConnect
 */

(function() {
    'use strict';

    const WALLET_PROVIDERS = {
        FREIGHTER: 'freighter',
        ALBEDO: 'albedo',
        WALLETCONNECT: 'walletconnect'
    };

    const STORAGE_KEYS = {
        CONNECTED_WALLET: 'stellar_connected_wallet',
        WALLET_ADDRESS: 'stellar_wallet_address',
        WALLET_PREFERENCES: 'stellar_wallet_preferences'
    };

    // Wallet state
    const walletState = {
        connected: false,
        provider: null,
        address: null,
        publicKey: null,
        network: 'testnet',
        preferences: {}
    };

    // ============================================
    // Wallet Detection & Loading
    // ============================================

    async function detectWallets() {
        const available = {};

        // Check Freighter
        if (window.freighter) {
            available.freighter = true;
        }

        // Check Albedo
        if (window.albedo) {
            available.albedo = true;
        }

        // WalletConnect always available
        available.walletconnect = true;

        return available;
    }

    async function loadFreighter() {
        try {
            if (!window.freighter) {
                throw new Error('Freighter wallet not installed');
            }

            const publicKey = await window.freighter.getPublicKey();
            if (!publicKey) throw new Error('Failed to get public key from Freighter');

            walletState.provider = WALLET_PROVIDERS.FREIGHTER;
            walletState.address = publicKey;
            walletState.publicKey = publicKey;
            walletState.connected = true;

            return { success: true, address: publicKey };
        } catch (error) {
            console.error('Freighter connection failed:', error);
            return { success: false, error: error.message };
        }
    }

    async function loadAlbedo() {
        try {
            if (!window.albedo) {
                throw new Error('Albedo wallet not installed');
            }

            const result = await window.albedo.publicKey();
            if (!result.publicKey) throw new Error('Failed to get public key from Albedo');

            walletState.provider = WALLET_PROVIDERS.ALBEDO;
            walletState.address = result.publicKey;
            walletState.publicKey = result.publicKey;
            walletState.connected = true;

            return { success: true, address: result.publicKey };
        } catch (error) {
            console.error('Albedo connection failed:', error);
            return { success: false, error: error.message };
        }
    }

    async function loadWalletConnect() {
        try {
            // Placeholder for WalletConnect integration
            // In production, integrate with @walletconnect/stellar-sdk
            console.log('WalletConnect integration requires additional setup');
            return { success: false, error: 'WalletConnect not yet configured' };
        } catch (error) {
            console.error('WalletConnect connection failed:', error);
            return { success: false, error: error.message };
        }
    }

    // ============================================
    // Connection Management
    // ============================================

    async function connectWallet(provider) {
        if (!provider || !Object.values(WALLET_PROVIDERS).includes(provider)) {
            return { success: false, error: 'Invalid wallet provider' };
        }

        let result;
        switch (provider) {
            case WALLET_PROVIDERS.FREIGHTER:
                result = await loadFreighter();
                break;
            case WALLET_PROVIDERS.ALBEDO:
                result = await loadAlbedo();
                break;
            case WALLET_PROVIDERS.WALLETCONNECT:
                result = await loadWalletConnect();
                break;
            default:
                return { success: false, error: 'Unknown provider' };
        }

        if (result.success) {
            saveWalletPreferences();
            dispatchWalletEvent('connected', { provider, address: result.address });
        }

        return result;
    }

    function disconnectWallet() {
        walletState.connected = false;
        walletState.provider = null;
        walletState.address = null;
        walletState.publicKey = null;

        localStorage.removeItem(STORAGE_KEYS.CONNECTED_WALLET);
        localStorage.removeItem(STORAGE_KEYS.WALLET_ADDRESS);

        dispatchWalletEvent('disconnected', {});
    }

    async function switchWallet(newProvider) {
        if (walletState.connected) {
            disconnectWallet();
        }
        return connectWallet(newProvider);
    }

    // ============================================
    // Transaction Signing
    // ============================================

    async function signTransaction(xdr) {
        if (!walletState.connected) {
            return { success: false, error: 'Wallet not connected' };
        }

        try {
            let signedXdr;

            switch (walletState.provider) {
                case WALLET_PROVIDERS.FREIGHTER:
                    signedXdr = await window.freighter.signTransaction(xdr, {
                        network: walletState.network
                    });
                    break;

                case WALLET_PROVIDERS.ALBEDO:
                    const albedoResult = await window.albedo.tx({
                        xdr: xdr,
                        network: walletState.network
                    });
                    signedXdr = albedoResult.signed_envelope_xdr;
                    break;

                default:
                    return { success: false, error: 'Signing not supported for this wallet' };
            }

            return { success: true, signedXdr };
        } catch (error) {
            console.error('Transaction signing failed:', error);
            return { success: false, error: error.message };
        }
    }

    // ============================================
    // Preferences Management
    // ============================================

    function saveWalletPreferences() {
        walletState.preferences = {
            lastConnected: new Date().toISOString(),
            provider: walletState.provider,
            network: walletState.network
        };

        localStorage.setItem(STORAGE_KEYS.CONNECTED_WALLET, walletState.provider);
        localStorage.setItem(STORAGE_KEYS.WALLET_ADDRESS, walletState.address);
        localStorage.setItem(STORAGE_KEYS.WALLET_PREFERENCES, JSON.stringify(walletState.preferences));
    }

    function loadWalletPreferences() {
        try {
            const provider = localStorage.getItem(STORAGE_KEYS.CONNECTED_WALLET);
            const address = localStorage.getItem(STORAGE_KEYS.WALLET_ADDRESS);
            const prefs = localStorage.getItem(STORAGE_KEYS.WALLET_PREFERENCES);

            if (provider && address) {
                walletState.provider = provider;
                walletState.address = address;
                walletState.publicKey = address;
                if (prefs) {
                    walletState.preferences = JSON.parse(prefs);
                }
                return true;
            }
        } catch (error) {
            console.error('Failed to load wallet preferences:', error);
        }
        return false;
    }

    function clearWalletPreferences() {
        localStorage.removeItem(STORAGE_KEYS.CONNECTED_WALLET);
        localStorage.removeItem(STORAGE_KEYS.WALLET_ADDRESS);
        localStorage.removeItem(STORAGE_KEYS.WALLET_PREFERENCES);
        walletState.preferences = {};
    }

    // ============================================
    // Event Dispatching
    // ============================================

    function dispatchWalletEvent(eventType, detail) {
        const event = new CustomEvent(`wallet:${eventType}`, {
            detail,
            bubbles: true,
            cancelable: true
        });
        document.dispatchEvent(event);
    }

    // ============================================
    // UI Helpers
    // ============================================

    function formatAddress(address) {
        if (!address) return '';
        return `${address.substring(0, 6)}...${address.substring(address.length - 6)}`;
    }

    function getProviderDisplayName(provider) {
        const names = {
            [WALLET_PROVIDERS.FREIGHTER]: 'Freighter',
            [WALLET_PROVIDERS.ALBEDO]: 'Albedo',
            [WALLET_PROVIDERS.WALLETCONNECT]: 'WalletConnect'
        };
        return names[provider] || 'Unknown';
    }

    // ============================================
    // Public API
    // ============================================

    window.StellarWallet = {
        // Connection
        connectWallet,
        disconnectWallet,
        switchWallet,
        detectWallets,

        // Transactions
        signTransaction,

        // Preferences
        saveWalletPreferences,
        loadWalletPreferences,
        clearWalletPreferences,

        // State
        getState: () => ({ ...walletState }),
        isConnected: () => walletState.connected,
        getAddress: () => walletState.address,
        getProvider: () => walletState.provider,

        // Utilities
        formatAddress,
        getProviderDisplayName,

        // Constants
        PROVIDERS: WALLET_PROVIDERS,
        STORAGE_KEYS
    };

    console.log('Stellar Wallet module loaded');
})();
