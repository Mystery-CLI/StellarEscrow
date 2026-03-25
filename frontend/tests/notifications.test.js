/**
 * Real-time Notifications Tests
 */

describe('Notification Manager', () => {
    
    beforeEach(() => {
        fetch.resetMocks();
    });

    describe('WebSocket Connection', () => {
        test('should connect to WebSocket', () => {
            window.NotificationManager.connectWebSocket();
            
            expect(window.NotificationManager.getState().connected).toBeDefined();
        });

        test('should disconnect from WebSocket', () => {
            window.NotificationManager.connectWebSocket();
            window.NotificationManager.disconnectWebSocket();
            
            expect(window.NotificationManager.getState().connected).toBe(false);
        });
    });

    describe('Preferences Management', () => {
        test('should load preferences', async () => {
            const mockPrefs = {
                tradeUpdates: true,
                disputes: true,
                arbitration: true,
                pushEnabled: false,
                emailEnabled: false
            };
            
            fetch.mockResponseOnce(JSON.stringify(mockPrefs));
            
            const result = await window.NotificationManager.loadPreferences();
            
            expect(result.success).toBe(true);
            expect(result.preferences.tradeUpdates).toBe(true);
        });

        test('should update preferences', async () => {
            fetch.mockResponseOnce(JSON.stringify({ success: true }));
            
            const result = await window.NotificationManager.updatePreferences({
                tradeUpdates: false,
                pushEnabled: true
            });
            
            expect(result.success).toBe(true);
            expect(fetch).toHaveBeenCalledWith(
                '/api/notifications/preferences',
                expect.objectContaining({ method: 'POST' })
            );
        });

        test('should handle preference update error', async () => {
            fetch.mockResponseOnce('', { status: 400 });
            
            const result = await window.NotificationManager.updatePreferences({});
            
            expect(result.success).toBe(false);
        });
    });

    describe('Push Notifications', () => {
        test('should request push permission', async () => {
            global.Notification = {
                permission: 'default',
                requestPermission: jest.fn().mockResolvedValue('granted')
            };
            
            const result = await window.NotificationManager.requestPushPermission();
            
            expect(result.success).toBe(true);
        });

        test('should handle push not supported', async () => {
            delete global.Notification;
            
            const result = await window.NotificationManager.requestPushPermission();
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('not supported');
        });
    });

    describe('State Management', () => {
        test('should get notification state', () => {
            const state = window.NotificationManager.getState();
            
            expect(state.connected).toBeDefined();
            expect(state.notifications).toBeDefined();
            expect(state.preferences).toBeDefined();
            expect(state.unreadCount).toBeDefined();
        });
    });

    describe('Notification Display', () => {
        test('should display notification center', () => {
            window.NotificationManager.displayNotificationCenter();
            
            const center = document.getElementById('notification-center');
            expect(center).toBeDefined();
        });
    });
});
