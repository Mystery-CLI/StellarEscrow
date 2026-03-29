/**
 * Wallet Integration Tests
 * Tests for Stellar wallet connection, switching, and preferences
 */

describe('Stellar Wallet Integration', () => {
    
    beforeEach(() => {
        // Clear localStorage before each test
        localStorage.clear();
        
        // Mock wallet providers
        window.freighter = {
            getPublicKey: jest.fn(),
            signTransaction: jest.fn()
        };
        
        window.albedo = {
            publicKey: jest.fn(),
            tx: jest.fn()
        };
    });

    afterEach(() => {
        delete window.freighter;
        delete window.albedo;
    });

    describe('Wallet Detection', () => {
        test('should detect available wallets', async () => {
            const available = await window.StellarWallet.detectWallets();
            
            expect(available.freighter).toBe(true);
            expect(available.albedo).toBe(true);
            expect(available.walletconnect).toBe(true);
        });

        test('should handle missing wallets', async () => {
            delete window.freighter;
            
            const available = await window.StellarWallet.detectWallets();
            
            expect(available.freighter).toBeFalsy();
            expect(available.albedo).toBe(true);
        });
    });

    describe('Wallet Connection', () => {
        test('should connect to Freighter wallet', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            const result = await window.StellarWallet.connectWallet('freighter');
            
            expect(result.success).toBe(true);
            expect(result.address).toBe(mockAddress);
            expect(window.StellarWallet.isConnected()).toBe(true);
            expect(window.StellarWallet.getProvider()).toBe('freighter');
        });

        test('should connect to Albedo wallet', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.albedo.publicKey.mockResolvedValue({ publicKey: mockAddress });
            
            const result = await window.StellarWallet.connectWallet('albedo');
            
            expect(result.success).toBe(true);
            expect(result.address).toBe(mockAddress);
            expect(window.StellarWallet.isConnected()).toBe(true);
            expect(window.StellarWallet.getProvider()).toBe('albedo');
        });

        test('should handle connection failure', async () => {
            window.freighter.getPublicKey.mockRejectedValue(new Error('User denied'));
            
            const result = await window.StellarWallet.connectWallet('freighter');
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('User denied');
            expect(window.StellarWallet.isConnected()).toBe(false);
        });

        test('should reject invalid provider', async () => {
            const result = await window.StellarWallet.connectWallet('invalid');
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('Invalid wallet provider');
        });
    });

    describe('Wallet Disconnection', () => {
        test('should disconnect wallet', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            await window.StellarWallet.connectWallet('freighter');
            expect(window.StellarWallet.isConnected()).toBe(true);
            
            window.StellarWallet.disconnectWallet();
            
            expect(window.StellarWallet.isConnected()).toBe(false);
            expect(window.StellarWallet.getAddress()).toBeNull();
            expect(window.StellarWallet.getProvider()).toBeNull();
        });

        test('should clear wallet from localStorage on disconnect', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            await window.StellarWallet.connectWallet('freighter');
            window.StellarWallet.disconnectWallet();
            
            expect(localStorage.getItem('stellar_connected_wallet')).toBeNull();
            expect(localStorage.getItem('stellar_wallet_address')).toBeNull();
        });
    });

    describe('Wallet Switching', () => {
        test('should switch between wallets', async () => {
            const freighterAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            const albedoAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXV';
            
            window.freighter.getPublicKey.mockResolvedValue(freighterAddress);
            window.albedo.publicKey.mockResolvedValue({ publicKey: albedoAddress });
            
            // Connect to Freighter
            await window.StellarWallet.connectWallet('freighter');
            expect(window.StellarWallet.getProvider()).toBe('freighter');
            
            // Switch to Albedo
            const result = await window.StellarWallet.switchWallet('albedo');
            
            expect(result.success).toBe(true);
            expect(window.StellarWallet.getProvider()).toBe('albedo');
            expect(window.StellarWallet.getAddress()).toBe(albedoAddress);
        });
    });

    describe('Wallet Preferences', () => {
        test('should save wallet preferences', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            await window.StellarWallet.connectWallet('freighter');
            window.StellarWallet.saveWalletPreferences();
            
            expect(localStorage.getItem('stellar_connected_wallet')).toBe('freighter');
            expect(localStorage.getItem('stellar_wallet_address')).toBe(mockAddress);
            
            const prefs = JSON.parse(localStorage.getItem('stellar_wallet_preferences'));
            expect(prefs.provider).toBe('freighter');
            expect(prefs.lastConnected).toBeDefined();
        });

        test('should load wallet preferences', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            
            localStorage.setItem('stellar_connected_wallet', 'freighter');
            localStorage.setItem('stellar_wallet_address', mockAddress);
            localStorage.setItem('stellar_wallet_preferences', JSON.stringify({
                provider: 'freighter',
                lastConnected: new Date().toISOString()
            }));
            
            const loaded = window.StellarWallet.loadWalletPreferences();
            
            expect(loaded).toBe(true);
            expect(window.StellarWallet.getProvider()).toBe('freighter');
            expect(window.StellarWallet.getAddress()).toBe(mockAddress);
        });

        test('should clear wallet preferences', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            await window.StellarWallet.connectWallet('freighter');
            window.StellarWallet.saveWalletPreferences();
            
            window.StellarWallet.clearWalletPreferences();
            
            expect(localStorage.getItem('stellar_connected_wallet')).toBeNull();
            expect(localStorage.getItem('stellar_wallet_address')).toBeNull();
            expect(localStorage.getItem('stellar_wallet_preferences')).toBeNull();
        });
    });

    describe('Transaction Signing', () => {
        test('should sign transaction with Freighter', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            const mockXdr = 'AAAAAgAAAABIW...';
            const mockSignedXdr = 'AAAAAgAAAABIW...SIGNED';
            
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            window.freighter.signTransaction.mockResolvedValue(mockSignedXdr);
            
            await window.StellarWallet.connectWallet('freighter');
            const result = await window.StellarWallet.signTransaction(mockXdr);
            
            expect(result.success).toBe(true);
            expect(result.signedXdr).toBe(mockSignedXdr);
            expect(window.freighter.signTransaction).toHaveBeenCalledWith(mockXdr, expect.any(Object));
        });

        test('should fail signing when wallet not connected', async () => {
            const mockXdr = 'AAAAAgAAAABIW...';
            
            const result = await window.StellarWallet.signTransaction(mockXdr);
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('Wallet not connected');
        });

        test('should handle signing errors', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            const mockXdr = 'AAAAAgAAAABIW...';
            
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            window.freighter.signTransaction.mockRejectedValue(new Error('User rejected'));
            
            await window.StellarWallet.connectWallet('freighter');
            const result = await window.StellarWallet.signTransaction(mockXdr);
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('User rejected');
        });
    });

    describe('Utility Functions', () => {
        test('should format address correctly', () => {
            const address = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            const formatted = window.StellarWallet.formatAddress(address);
            
            expect(formatted).toBe('GBRPYH...GBXU');
            expect(formatted.length).toBeLessThan(address.length);
        });

        test('should get provider display name', () => {
            expect(window.StellarWallet.getProviderDisplayName('freighter')).toBe('Freighter');
            expect(window.StellarWallet.getProviderDisplayName('albedo')).toBe('Albedo');
            expect(window.StellarWallet.getProviderDisplayName('walletconnect')).toBe('WalletConnect');
            expect(window.StellarWallet.getProviderDisplayName('unknown')).toBe('Unknown');
        });

        test('should get wallet state', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            await window.StellarWallet.connectWallet('freighter');
            const state = window.StellarWallet.getState();
            
            expect(state.connected).toBe(true);
            expect(state.provider).toBe('freighter');
            expect(state.address).toBe(mockAddress);
        });
    });

    describe('Events', () => {
        test('should dispatch wallet:connected event', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            const eventListener = jest.fn();
            document.addEventListener('wallet:connected', eventListener);
            
            await window.StellarWallet.connectWallet('freighter');
            
            expect(eventListener).toHaveBeenCalled();
            expect(eventListener.mock.calls[0][0].detail.provider).toBe('freighter');
            expect(eventListener.mock.calls[0][0].detail.address).toBe(mockAddress);
        });

        test('should dispatch wallet:disconnected event', async () => {
            const mockAddress = 'GBRPYHIL2CI3WHZDTOOQFC6EB4KJJGUJJBBX7XNLG5BNXKYL2AQGBXU';
            window.freighter.getPublicKey.mockResolvedValue(mockAddress);
            
            await window.StellarWallet.connectWallet('freighter');
            
            const eventListener = jest.fn();
            document.addEventListener('wallet:disconnected', eventListener);
            
            window.StellarWallet.disconnectWallet();
            
            expect(eventListener).toHaveBeenCalled();
        });
    });
});
