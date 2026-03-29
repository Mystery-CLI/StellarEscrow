/**
 * Real-time Notifications System
 * WebSocket-based notifications for trades, disputes, and events
 */

(function() {
    'use strict';

    const NOTIFICATION_API = '/api/notifications';
    const WS_PROTOCOL = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const WS_URL = `${WS_PROTOCOL}//${window.location.host}/api/notifications/ws`;

    const notificationState = {
        connected: false,
        socket: null,
        notifications: [],
        preferences: {
            tradeUpdates: true,
            disputes: true,
            arbitration: true,
            pushEnabled: false,
            emailEnabled: false,
            emailAddress: null
        },
        unreadCount: 0
    };

    // ============================================
    // WebSocket Management
    // ============================================

    function connectWebSocket() {
        try {
            notificationState.socket = new WebSocket(WS_URL);

            notificationState.socket.onopen = () => {
                notificationState.connected = true;
                dispatchNotificationEvent('connected', {});
                console.log('Notification WebSocket connected');
            };

            notificationState.socket.onmessage = (event) => {
                try {
                    const notification = JSON.parse(event.data);
                    handleNotification(notification);
                } catch (error) {
                    console.error('Failed to parse notification:', error);
                }
            };

            notificationState.socket.onclose = () => {
                notificationState.connected = false;
                dispatchNotificationEvent('disconnected', {});
                setTimeout(connectWebSocket, 3000);
            };

            notificationState.socket.onerror = (error) => {
                console.error('WebSocket error:', error);
                notificationState.connected = false;
            };
        } catch (error) {
            console.error('Failed to create WebSocket:', error);
        }
    }

    function disconnectWebSocket() {
        if (notificationState.socket) {
            notificationState.socket.close();
            notificationState.socket = null;
            notificationState.connected = false;
        }
    }

    // ============================================
    // Notification Handling
    // ============================================

    function handleNotification(notification) {
        // Check preferences
        if (!shouldShowNotification(notification)) {
            return;
        }

        // Add to notifications list
        notificationState.notifications.unshift({
            ...notification,
            id: Date.now(),
            read: false,
            timestamp: new Date().toISOString()
        });

        // Limit notifications
        if (notificationState.notifications.length > 100) {
            notificationState.notifications.pop();
        }

        notificationState.unreadCount++;

        // Show UI notification
        displayNotification(notification);

        // Send push notification
        if (notificationState.preferences.pushEnabled) {
            sendPushNotification(notification);
        }

        // Send email notification
        if (notificationState.preferences.emailEnabled) {
            sendEmailNotification(notification);
        }

        dispatchNotificationEvent('received', notification);
    }

    function shouldShowNotification(notification) {
        switch (notification.type) {
            case 'trade_updated':
                return notificationState.preferences.tradeUpdates;
            case 'dispute_raised':
            case 'dispute_resolved':
                return notificationState.preferences.disputes;
            case 'arbitration_assigned':
                return notificationState.preferences.arbitration;
            default:
                return true;
        }
    }

    function displayNotification(notification) {
        const container = document.getElementById('notification-toast-container');
        if (!container) return;

        const toast = document.createElement('div');
        toast.className = `notification-toast notification-${notification.type}`;
        toast.setAttribute('role', 'alert');
        toast.setAttribute('aria-live', 'assertive');

        const icon = getNotificationIcon(notification.type);
        const title = getNotificationTitle(notification.type);

        toast.innerHTML = `
            <div class="notification-content">
                <span class="notification-icon">${icon}</span>
                <div class="notification-text">
                    <strong>${title}</strong>
                    <p>${escapeHtml(notification.message)}</p>
                </div>
                <button 
                    type="button"
                    class="notification-close"
                    aria-label="Close notification"
                >
                    ✕
                </button>
            </div>
        `;

        container.appendChild(toast);

        // Close button
        toast.querySelector('.notification-close').addEventListener('click', () => {
            toast.style.opacity = '0';
            setTimeout(() => toast.remove(), 300);
        });

        // Auto-dismiss
        setTimeout(() => {
            if (toast.parentNode) {
                toast.style.opacity = '0';
                setTimeout(() => toast.remove(), 300);
            }
        }, 5000);
    }

    // ============================================
    // Push Notifications
    // ============================================

    async function sendPushNotification(notification) {
        if (!('Notification' in window)) return;

        if (Notification.permission === 'granted') {
            new Notification(getNotificationTitle(notification.type), {
                body: notification.message,
                icon: '/icon.png',
                tag: notification.type
            });
        }
    }

    async function requestPushPermission() {
        if (!('Notification' in window)) {
            return { success: false, error: 'Push notifications not supported' };
        }

        if (Notification.permission === 'granted') {
            return { success: true, granted: true };
        }

        if (Notification.permission !== 'denied') {
            const permission = await Notification.requestPermission();
            return { success: true, granted: permission === 'granted' };
        }

        return { success: false, error: 'Push notifications denied' };
    }

    // ============================================
    // Email Notifications
    // ============================================

    async function sendEmailNotification(notification) {
        if (!notificationState.preferences.emailAddress) return;

        try {
            await fetch(`${NOTIFICATION_API}/email`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    email: notificationState.preferences.emailAddress,
                    subject: getNotificationTitle(notification.type),
                    message: notification.message,
                    type: notification.type
                })
            });
        } catch (error) {
            console.error('Failed to send email notification:', error);
        }
    }

    // ============================================
    // Preferences Management
    // ============================================

    async function updatePreferences(preferences) {
        try {
            const response = await fetch(`${NOTIFICATION_API}/preferences`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(preferences)
            });

            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            notificationState.preferences = { ...notificationState.preferences, ...preferences };
            return { success: true };
        } catch (error) {
            console.error('Failed to update preferences:', error);
            return { success: false, error: error.message };
        }
    }

    async function loadPreferences() {
        try {
            const response = await fetch(`${NOTIFICATION_API}/preferences`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            const prefs = await response.json();
            notificationState.preferences = { ...notificationState.preferences, ...prefs };
            return { success: true, preferences: prefs };
        } catch (error) {
            console.error('Failed to load preferences:', error);
            return { success: false, error: error.message };
        }
    }

    // ============================================
    // UI Initialization
    // ============================================

    function initNotificationUI() {
        const prefsBtn = document.getElementById('notification-prefs-btn');
        const modal = document.getElementById('notification-prefs-modal');
        const form = document.getElementById('notification-prefs-form');
        const closeBtn = document.getElementById('notification-prefs-close');

        if (!prefsBtn) return;

        // Open preferences
        prefsBtn.addEventListener('click', () => {
            if (modal) modal.hidden = false;
            loadPreferencesForm();
        });

        // Close preferences
        if (closeBtn) {
            closeBtn.addEventListener('click', () => {
                if (modal) modal.hidden = true;
            });
        }

        // Save preferences
        if (form) {
            form.addEventListener('submit', async (e) => {
                e.preventDefault();

                const prefs = {
                    tradeUpdates: document.getElementById('pref-trade-updates').checked,
                    disputes: document.getElementById('pref-disputes').checked,
                    arbitration: document.getElementById('pref-arbitration').checked,
                    pushEnabled: document.getElementById('pref-push').checked,
                    emailEnabled: document.getElementById('pref-email').checked,
                    emailAddress: document.getElementById('pref-email-address').value
                };

                const result = await updatePreferences(prefs);
                if (result.success) {
                    showToast('Preferences updated', 'success');
                    if (modal) modal.hidden = true;
                } else {
                    showToast('Failed to update preferences', 'error');
                }
            });
        }

        // Request push permission
        const pushBtn = document.getElementById('request-push-permission');
        if (pushBtn) {
            pushBtn.addEventListener('click', async () => {
                const result = await requestPushPermission();
                if (result.success && result.granted) {
                    showToast('Push notifications enabled', 'success');
                    document.getElementById('pref-push').checked = true;
                } else {
                    showToast('Push notifications denied', 'error');
                }
            });
        }
    }

    function loadPreferencesForm() {
        document.getElementById('pref-trade-updates').checked = notificationState.preferences.tradeUpdates;
        document.getElementById('pref-disputes').checked = notificationState.preferences.disputes;
        document.getElementById('pref-arbitration').checked = notificationState.preferences.arbitration;
        document.getElementById('pref-push').checked = notificationState.preferences.pushEnabled;
        document.getElementById('pref-email').checked = notificationState.preferences.emailEnabled;
        document.getElementById('pref-email-address').value = notificationState.preferences.emailAddress || '';
    }

    function displayNotificationCenter() {
        const container = document.getElementById('notification-center');
        if (!container) return;

        if (notificationState.notifications.length === 0) {
            container.innerHTML = '<p class="empty-state">No notifications</p>';
            return;
        }

        container.innerHTML = notificationState.notifications.map(n => `
            <div class="notification-item ${n.read ? 'read' : 'unread'}">
                <div class="notification-header">
                    <span class="notification-type">${getNotificationTitle(n.type)}</span>
                    <span class="notification-time">${formatTimestamp(n.timestamp)}</span>
                </div>
                <p class="notification-message">${escapeHtml(n.message)}</p>
            </div>
        `).join('');
    }

    // ============================================
    // Utilities
    // ============================================

    function getNotificationIcon(type) {
        const icons = {
            'trade_updated': '📦',
            'trade_completed': '✅',
            'dispute_raised': '⚠️',
            'dispute_resolved': '✓',
            'arbitration_assigned': '⚖️',
            'payment_received': '💰'
        };
        return icons[type] || '📢';
    }

    function getNotificationTitle(type) {
        const titles = {
            'trade_updated': 'Trade Updated',
            'trade_completed': 'Trade Completed',
            'dispute_raised': 'Dispute Raised',
            'dispute_resolved': 'Dispute Resolved',
            'arbitration_assigned': 'Arbitration Assigned',
            'payment_received': 'Payment Received'
        };
        return titles[type] || 'Notification';
    }

    function formatTimestamp(timestamp) {
        const date = new Date(timestamp);
        const now = new Date();
        const diff = now - date;

        if (diff < 60000) return 'Just now';
        if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
        if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
        return date.toLocaleDateString();
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function showToast(message, type = 'info') {
        const container = document.getElementById('toast-container');
        if (!container) return;

        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.setAttribute('role', 'alert');
        toast.textContent = message;
        container.appendChild(toast);

        setTimeout(() => {
            toast.style.opacity = '0';
            setTimeout(() => toast.remove(), 300);
        }, 5000);
    }

    function dispatchNotificationEvent(eventType, detail) {
        const event = new CustomEvent(`notification:${eventType}`, {
            detail,
            bubbles: true,
            cancelable: true
        });
        document.dispatchEvent(event);
    }

    // ============================================
    // Public API
    // ============================================

    window.NotificationManager = {
        connectWebSocket,
        disconnectWebSocket,
        updatePreferences,
        loadPreferences,
        requestPushPermission,
        displayNotificationCenter,
        initNotificationUI,
        getState: () => ({ ...notificationState })
    };

    console.log('Notification Manager module loaded');
})();
