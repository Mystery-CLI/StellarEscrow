/**
 * StellarEscrow Dashboard - Accessible JavaScript Application
 * WCAG 2.1 AA Compliance Implementation
 */

import { setLocale, getLocale, formatCurrency, formatDate } from './i18n.js';
import { registerServiceWorker, initOfflineIndicator, promptInstall, isInstallable, subscribePush } from './pwa.js';
import { observeWebVitals, initLazyRoutes, prefetchOnIdle, cachedFetch, invalidateCache } from './performance.js';
import { initErrorBoundary, reportError, friendlyMessage, withRetry, showErrorUI, logError } from './error-handler.js';

(function() {
    'use strict';

    // ============================================
    // Configuration
    // ============================================
    const CONFIG = {
        apiBaseUrl: '/api',
        wsReconnectDelay: 3000,
        maxEventsInFeed: 100,
        searchDebounceDelay: 300
    };

    // ============================================
    // State Management
    // ============================================
    const state = {
        events: [],
        currentPage: 1,
        totalEvents: 0,
        pageSize: 50,
        wsConnected: false,
        wsSocket: null,
        searchHistory: [],
        highContrastMode: false,
        focusedElement: null,
        walletConnected: false,
        walletProvider: null,
        walletAddress: null
    };

    // ============================================
    // DOM Utilities
    // ============================================
    const $ = (selector) => document.querySelector(selector);
    const $$ = (selector) => document.querySelectorAll(selector);

    /**
     * Announce message to screen readers via live region
     */
    function announce(message, priority = 'polite') {
        const liveRegion = $('#live-region');
        if (liveRegion) {
            liveRegion.setAttribute('aria-live', priority);
            liveRegion.textContent = '';
            // Small delay to ensure screen reader picks up the change
            requestAnimationFrame(() => {
                liveRegion.textContent = message;
            });
        }
    }

    /**
     * Show toast notification
     */
    function showToast(message, type = 'info') {
        const container = $('#toast-container');
        if (!container) return;

        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.setAttribute('role', 'alert');
        toast.setAttribute('aria-live', 'assertive');
        toast.textContent = message;
        container.appendChild(toast);

        // Remove after 5 seconds
        setTimeout(() => {
            toast.style.opacity = '0';
            setTimeout(() => toast.remove(), 300);
        }, 5000);
    }

    /**
     * Format timestamp for display
     */
    function formatTimestamp(timestamp) {
        const date = new Date(timestamp);
        return date.toLocaleString('en-US', {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        });
    }

    /**
     * Escape HTML to prevent XSS
     */
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // ============================================
    // API Functions
    // ============================================
    async function fetchHealth() {
        try {
            const data = await cachedFetch(`${CONFIG.apiBaseUrl}/health`, {}, 60_000);
            updateHealthStatus(data);
            return data;
        } catch (error) {
            console.error('Health check failed:', error);
            updateHealthStatus({ status: 'error', error: error.message });
            return null;
        }
    }

    async function fetchEvents(params = {}) {
        const queryParams = new URLSearchParams();
        
        if (params.event_type) queryParams.set('event_type', params.event_type);
        if (params.trade_id) queryParams.set('trade_id', params.trade_id);
        queryParams.set('limit', params.limit || state.pageSize);
        queryParams.set('offset', (state.currentPage - 1) * state.pageSize);

        try {
            const data = await withRetry(
                () => cachedFetch(`${CONFIG.apiBaseUrl}/events?${queryParams}`, {}, 15_000),
                {
                    maxAttempts: 3,
                    onRetry: (attempt) => showToast(`Retrying… (${attempt}/3)`, 'warning'),
                }
            );
            state.events = data;
            state.totalEvents = data.length;
            renderEventsTable(data);
            announce(`Loaded ${data.length} events`);
            return data;
        } catch (error) {
            logError('Failed to fetch events', { error: error?.message });
            reportError(error, { context: 'fetchEvents' });
            const msg = friendlyMessage(error);
            showErrorUI(msg, { retryFn: () => fetchEvents(params) });
            announce(msg, 'assertive');
            return [];
        }
    }

    async function fetchHelpContent(type) {
        try {
            const endpoint = {
                'faqs': '/api/help/faqs',
                'tutorials': '/api/help/tutorials',
                'docs': '/api/help/docs',
                'contact': '/api/help/contact'
            }[type];

            if (!endpoint) return null;

            const response = await fetch(endpoint);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return await response.json();
        } catch (error) {
            console.error(`Failed to fetch ${type}:`, error);
            return null;
        }
    }

    async function searchGlobal(query) {
        if (!query || query.trim().length === 0) return null;

        try {
            const response = await fetch(`${CONFIG.apiBaseUrl}/search?q=${encodeURIComponent(query)}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            const data = await response.json();
            renderSearchResults(data);
            addToSearchHistory(query);
            announce(`Found ${data.trades?.length || 0} trades, ${data.users?.length || 0} users`);
            return data;
        } catch (error) {
            console.error('Search failed:', error);
            showToast('Search failed', 'error');
            return null;
        }
    }

    // ============================================
    // WebSocket Connection
    // ============================================
    function connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/ws`;

        try {
            state.wsSocket = new WebSocket(wsUrl);

            state.wsSocket.onopen = () => {
                state.wsConnected = true;
                updateWsStatus(true);
                announce('WebSocket connected');
            };

            state.wsSocket.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    addEventToFeed(data);
                } catch (error) {
                    console.error('Failed to parse WebSocket message:', error);
                }
            };

            state.wsSocket.onclose = () => {
                state.wsConnected = false;
                updateWsStatus(false);
                // Attempt to reconnect
                setTimeout(connectWebSocket, CONFIG.wsReconnectDelay);
            };

            state.wsSocket.onerror = (error) => {
                console.error('WebSocket error:', error);
                state.wsConnected = false;
                updateWsStatus(false);
            };
        } catch (error) {
            console.error('Failed to create WebSocket:', error);
            state.wsConnected = false;
            updateWsStatus(false);
        }
    }

    function addEventToFeed(event) {
        const feed = $('#events-list');
        if (!feed) return;

        // Remove empty state if present
        const emptyState = feed.querySelector('.empty-state');
        if (emptyState) emptyState.remove();

        // Create event item
        const item = document.createElement('div');
        item.className = 'event-item';
        item.setAttribute('role', 'article');
        item.setAttribute('aria-label', `${event.event_type} event for trade ${event.trade_id || 'unknown'}`);
        
        item.innerHTML = `
            <span class="event-time">${formatTimestamp(event.timestamp)}</span>
            <span class="event-type">${escapeHtml(event.event_type || 'unknown')}</span>
            ${event.trade_id ? `<span class="event-trade-id">Trade #${event.trade_id}</span>` : ''}
        `;

        // Add to top of feed
        feed.insertBefore(item, feed.firstChild);

        // Limit feed size
        while (feed.children.length > CONFIG.maxEventsInFeed) {
            feed.removeChild(feed.lastChild);
        }

        // Update total count
        const totalEl = $('#total-events');
        if (totalEl) {
            const current = parseInt(totalEl.textContent) || 0;
            totalEl.textContent = current + 1;
        }
    }

    // ============================================
    // UI Updates
    // ============================================
    function updateHealthStatus(data) {
        const indicator = $('#health-indicator');
        if (!indicator) return;

        const dot = indicator.querySelector('.indicator-dot');
        const text = indicator.querySelector('.indicator-text');

        if (data.status === 'healthy') {
            dot?.classList.remove('error');
            text.textContent = 'Healthy';
        } else {
            dot?.classList.add('error');
            text.textContent = 'Error';
        }
    }

    function updateWsStatus(connected) {
        const indicator = $('#ws-indicator');
        if (!indicator) return;

        const dot = indicator.querySelector('.indicator-dot');
        const text = indicator.querySelector('.indicator-text');

        state.wsConnected = connected;
        
        if (connected) {
            dot?.classList.remove('disconnected');
            text.textContent = 'Connected';
        } else {
            dot?.classList.add('disconnected');
            text.textContent = 'Disconnected';
        }
    }

    function renderEventsTable(events) {
        const tbody = $('#events-table-body');
        if (!tbody) return;

        if (events.length === 0) {
            tbody.innerHTML = `
                <tr>
                    <td colspan="4" class="empty-state">No events found</td>
                </tr>
            `;
            return;
        }

        tbody.innerHTML = events.map(event => `
            <tr tabindex="0">
                <td>${formatTimestamp(event.timestamp)}</td>
                <td><span class="event-type">${escapeHtml(event.event_type || 'unknown')}</span></td>
                <td>${event.trade_id || '-'}</td>
                <td>${escapeHtml(JSON.stringify(event.data || {}))}</td>
            </tr>
        `).join('');
    }

    function renderSearchResults(data) {
        const container = $('#search-results');
        if (!container) return;

        let html = '';

        // Trades
        if (data.trades?.length > 0) {
            html += '<h3>Trades</h3>';
            data.trades.forEach(trade => {
                html += `
                    <article class="result-card" tabindex="0">
                        <h4 class="result-title">Trade #${trade.id}</h4>
                        <p class="result-meta">Status: ${escapeHtml(trade.status || 'unknown')}</p>
                    </article>
                `;
            });
        }

        // Users
        if (data.users?.length > 0) {
            html += '<h3>Users</h3>';
            data.users.forEach(user => {
                html += `
                    <article class="result-card" tabindex="0">
                        <h4 class="result-title">${escapeHtml(user.address || 'Unknown')}</h4>
                        <p class="result-meta">Role: ${escapeHtml(user.role || 'user')}</p>
                    </article>
                `;
            });
        }

        // Arbitrators
        if (data.arbitrators?.length > 0) {
            html += '<h3>Arbitrators</h3>';
            data.arbitrators.forEach(arb => {
                html += `
                    <article class="result-card" tabindex="0">
                        <h4 class="result-title">${escapeHtml(arb.address || 'Unknown')}</h4>
                        <p class="result-meta">Registered Arbitrator</p>
                    </article>
                `;
            });
        }

        if (!html) {
            html = '<p class="empty-state">No results found</p>';
        }

        container.innerHTML = html;
    }

    async function renderFAQs() {
        const container = $('#faqs-content .faq-list');
        if (!container) return;

        const data = await fetchHelpContent('faqs');
        if (!data) {
            container.innerHTML = '<p class="empty-state">Failed to load FAQs</p>';
            return;
        }

        container.innerHTML = data.map(faq => `
            <details class="faq-item">
                <button class="faq-question" aria-expanded="false">
                    ${escapeHtml(faq.question)}
                </button>
                <div class="faq-answer">
                    <p>${escapeHtml(faq.answer)}</p>
                </div>
            </details>
        `).join('');

        // Add event listeners for FAQ toggles
        container.querySelectorAll('.faq-question').forEach(btn => {
            btn.addEventListener('click', () => {
                const expanded = btn.getAttribute('aria-expanded') === 'true';
                btn.setAttribute('aria-expanded', !expanded);
            });
        });
    }

    async function renderTutorials() {
        const container = $('#tutorials-content .tutorial-list');
        if (!container) return;

        const data = await fetchHelpContent('tutorials');
        if (!data) {
            container.innerHTML = '<p class="empty-state">Failed to load tutorials</p>';
            return;
        }

        container.innerHTML = data.map(tutorial => `
            <article class="tutorial-card" tabindex="0">
                <h3 class="tutorial-title">${escapeHtml(tutorial.title)}</h3>
                <p class="tutorial-description">${escapeHtml(tutorial.description)}</p>
                <span class="tutorial-difficulty">${escapeHtml(tutorial.difficulty)}</span>
            </article>
        `).join('');
    }

    // ============================================
    // Search History
    // ============================================
    function addToSearchHistory(query) {
        if (!query || state.searchHistory.includes(query)) return;
        
        state.searchHistory.unshift(query);
        if (state.searchHistory.length > 10) {
            state.searchHistory.pop();
        }
        renderSearchHistory();
        localStorage.setItem('stellarEscrowSearchHistory', JSON.stringify(state.searchHistory));
    }

    function loadSearchHistory() {
        try {
            const stored = localStorage.getItem('stellarEscrowSearchHistory');
            if (stored) {
                state.searchHistory = JSON.parse(stored);
                renderSearchHistory();
            }
        } catch (error) {
            console.error('Failed to load search history:', error);
        }
    }

    function renderSearchHistory() {
        const list = $('#search-history-list');
        if (!list || state.searchHistory.length === 0) return;

        list.innerHTML = state.searchHistory.map(query => `
            <li class="history-item">
                <button type="button" class="history-btn" data-query="${escapeHtml(query)}">
                    ${escapeHtml(query)}
                </button>
            </li>
        `).join('');

        // Add click handlers
        list.querySelectorAll('.history-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const input = $('#search-input');
                if (input) {
                    input.value = btn.dataset.query;
                    $('#search-form').dispatchEvent(new Event('submit'));
                }
            });
        });
    }

    // ============================================
    // High Contrast Mode
    // ============================================
    function toggleHighContrast() {
        state.highContrastMode = !state.highContrastMode;
        document.documentElement.setAttribute('data-theme', 
            state.highContrastMode ? 'high-contrast' : '');
        
        const toggle = $('#contrast-toggle');
        if (toggle) {
            toggle.setAttribute('aria-pressed', state.highContrastMode);
            toggle.querySelector('.toggle-text').textContent = 
                state.highContrastMode ? 'Normal Mode' : 'High Contrast';
        }

        // Persist preference
        localStorage.setItem('highContrastMode', state.highContrastMode);
        
        // Update theme-color meta tag
        updateThemeColor();
        
        announce(state.highContrastMode ? 'High contrast mode enabled' : 'Normal mode enabled');
    }

    /**
     * Update theme-color meta tag based on high contrast mode
     */
    function updateThemeColor() {
        const metaThemeColor = document.querySelector('meta[name="theme-color"]');
        if (metaThemeColor) {
            metaThemeColor.setAttribute('content', 
                state.highContrastMode ? '#000000' : '#1a1a2e');
        }
    }

    function loadHighContrastPreference() {
        try {
            const stored = localStorage.getItem('highContrastMode');
            if (stored === 'true') {
                state.highContrastMode = true;
                document.documentElement.setAttribute('data-theme', 'high-contrast');
                const toggle = $('#contrast-toggle');
                if (toggle) {
                    toggle.setAttribute('aria-pressed', 'true');
                    toggle.querySelector('.toggle-text').textContent = 'Normal Mode';
                }
            }
        } catch (error) {
            console.error('Failed to load contrast preference:', error);
        }
    }

    // ============================================
    // Keyboard Navigation
    // ============================================
    function initKeyboardNavigation() {
        // Tab trap for modals (if any)
        document.addEventListener('keydown', (e) => {
            // Escape key to close any open modals/dropdowns
            if (e.key === 'Escape') {
                const openDetails = document.querySelector('details[open]');
                if (openDetails) {
                    openDetails.removeAttribute('open');
                }
            }

            // Enter/Space on FAQ questions
            if (e.key === 'Enter' || e.key === ' ') {
                const faqBtn = e.target.closest('.faq-question');
                if (faqBtn) {
                    e.preventDefault();
                    const details = faqBtn.closest('details');
                    if (details) {
                        details.hasAttribute('open') 
                            ? details.removeAttribute('open')
                            : details.setAttribute('open', '');
                        faqBtn.setAttribute('aria-expanded', details.hasAttribute('open'));
                    }
                }
            }
        });

        // Arrow key navigation in tables
        document.addEventListener('keydown', (e) => {
            const table = e.target.closest('.data-table');
            if (!table) return;

            const cells = table.querySelectorAll('tbody td');
            const currentCell = e.target.closest('td');
            if (!currentCell) return;

            const currentIndex = Array.from(cells).indexOf(currentCell);
            
            switch(e.key) {
                case 'ArrowUp':
                    e.preventDefault();
                    if (currentIndex >= 4) cells[currentIndex - 4]?.focus();
                    break;
                case 'ArrowDown':
                    e.preventDefault();
                    if (currentIndex + 4 < cells.length) cells[currentIndex + 4]?.focus();
                    break;
                case 'ArrowLeft':
                    e.preventDefault();
                    if (currentIndex > 0) cells[currentIndex - 1]?.focus();
                    break;
                case 'ArrowRight':
                    e.preventDefault();
                    if (currentIndex + 1 < cells.length) cells[currentIndex + 1]?.focus();
                    break;
            }
        });

        // Global keyboard shortcut for high contrast toggle (Alt+H)
        document.addEventListener('keydown', (e) => {
            if (e.altKey && e.key.toLowerCase() === 'h') {
                e.preventDefault();
                toggleHighContrast();
            }
        });
    }

    // ============================================
    // Mobile Menu
    // ============================================
    function initMobileMenu() {
        const menuBtn = $('#mobile-menu-btn');
        const menu = $('#main-menu');

        if (!menuBtn || !menu) return;

        menuBtn.addEventListener('click', () => {
            const isOpen = menu.classList.toggle('open');
            menuBtn.setAttribute('aria-expanded', isOpen);
            announce(isOpen ? 'Menu opened' : 'Menu closed');
        });

        // Close menu when clicking outside
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.nav-container')) {
                menu.classList.remove('open');
                menuBtn.setAttribute('aria-expanded', 'false');
            }
        });
    }

    // ============================================
    // Help Navigation
    // ============================================
    function initHelpNav() {
        const navBtns = $$('.help-nav-btn');
        const contents = $$('.help-content');

        navBtns.forEach(btn => {
            btn.addEventListener('click', () => {
                const target = btn.dataset.target;
                
                // Update button states
                navBtns.forEach(b => {
                    b.classList.remove('active');
                    b.setAttribute('aria-pressed', 'false');
                });
                btn.classList.add('active');
                btn.setAttribute('aria-pressed', 'true');

                // Update content visibility
                contents.forEach(content => {
                    if (content.id === `${target}-content`) {
                        content.classList.add('active');
                        content.hidden = false;
                    } else {
                        content.classList.remove('active');
                        content.hidden = true;
                    }
                });

                announce(`${btn.textContent} section`);
            });
        });
    }

    // ============================================
    // Form Handlers
    // ============================================
    function initForms() {
        // Events filter form
        const eventsForm = $('#events-filter-form');
        if (eventsForm) {
            eventsForm.addEventListener('submit', (e) => {
                e.preventDefault();
                state.currentPage = 1;
                
                const params = {
                    event_type: $('#event-type-filter')?.value,
                    trade_id: $('#trade-id-filter')?.value,
                    limit: $('#limit-select')?.value
                };
                
                fetchEvents(params);
            });
        }

        // Pagination
        $$('.pagination-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                if (btn.dataset.action === 'prev' && state.currentPage > 1) {
                    state.currentPage--;
                    fetchEvents();
                } else if (btn.dataset.action === 'next') {
                    state.currentPage++;
                    fetchEvents();
                }
            });
        });

        // Search form
        let searchTimeout;
        const searchForm = $('#search-form');
        const searchInput = $('#search-input');

        if (searchForm && searchInput) {
            searchForm.addEventListener('submit', (e) => {
                e.preventDefault();
                const query = searchInput.value.trim();
                if (query) {
                    searchGlobal(query);
                }
            });

            // Debounced search on input
            searchInput.addEventListener('input', () => {
                clearTimeout(searchTimeout);
                searchTimeout = setTimeout(() => {
                    if (searchInput.value.length >= 3) {
                        searchGlobal(searchInput.value);
                    }
                }, CONFIG.searchDebounceDelay);
            });
        }

        // Clear feed button
        const clearFeedBtn = $('#clear-feed-btn');
        if (clearFeedBtn) {
            clearFeedBtn.addEventListener('click', () => {
                const feed = $('#events-list');
                if (feed) {
                    feed.innerHTML = '<p class="empty-state">Feed cleared</p>';
                    announce('Events feed cleared');
                }
            });
        }
    }

    // ============================================
    // Smooth Scroll
    // ============================================
    function initSmoothScroll() {
        document.querySelectorAll('a[href^="#"]').forEach(anchor => {
            anchor.addEventListener('click', (e) => {
                const targetId = anchor.getAttribute('href');
                if (targetId === '#') return;
                
                const target = document.querySelector(targetId);
                if (target) {
                    e.preventDefault();
                    target.scrollIntoView({ behavior: 'smooth' });
                    
                    // Update focus for accessibility
                    target.setAttribute('tabindex', '-1');
                    target.focus({ preventScroll: true });
                }
            });
        });
    }

    // ============================================
    // Focus Management
    // ============================================
    function initFocusManagement() {
        // Track last focused element
        document.addEventListener('focusin', (e) => {
            state.focusedElement = e.target;
        });

        // Restore focus after page hide/show
        document.addEventListener('visibilitychange', () => {
            if (!document.hidden && state.focusedElement) {
                state.focusedElement.focus();
            }
        });
    }

    // ============================================
    // Reduced Motion Preference
    // ============================================
    function checkReducedMotion() {
        const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
        
        prefersReducedMotion.addEventListener('change', (e) => {
            if (e.matches) {
                document.documentElement.style.setProperty('--transition-fast', '0ms');
                document.documentElement.style.setProperty('--transition-normal', '0ms');
                announce('Reduced motion enabled');
            } else {
                document.documentElement.style.setProperty('--transition-fast', '150ms ease');
                document.documentElement.style.setProperty('--transition-normal', '250ms ease');
                announce('Reduced motion disabled');
            }
        });

        if (prefersReducedMotion.matches) {
            document.documentElement.style.setProperty('--transition-fast', '0ms');
            document.documentElement.style.setProperty('--transition-normal', '0ms');
        }
    }

    // ============================================
    // Wallet Integration
    // ============================================
    function initWalletUI() {
        const connectBtn = $('#wallet-connect-btn');
        const walletMenu = $('#wallet-menu');
        const walletConnected = $('#wallet-connected');
        const walletOptions = $$('.wallet-option');
        const disconnectBtn = $('#disconnect-wallet-btn');
        const switchWalletBtn = $('#switch-wallet-btn');
        const walletMenuToggle = $('#wallet-menu-toggle');
        const walletDropdown = $('#wallet-dropdown');

        if (!connectBtn) return;

        // Toggle wallet menu
        connectBtn.addEventListener('click', () => {
            const isOpen = !walletMenu.hidden;
            walletMenu.hidden = isOpen;
            connectBtn.setAttribute('aria-expanded', !isOpen);
            if (!isOpen) {
                announce('Wallet menu opened');
            }
        });

        // Wallet option selection
        walletOptions.forEach(option => {
            option.addEventListener('click', async () => {
                const provider = option.dataset.provider;
                connectBtn.disabled = true;
                connectBtn.textContent = 'Connecting...';
                announce(`Connecting to ${provider} wallet...`);

                const result = await window.StellarWallet.connectWallet(provider);
                
                if (result.success) {
                    updateWalletUI(result.address, provider);
                    walletMenu.hidden = true;
                    connectBtn.setAttribute('aria-expanded', 'false');
                    showToast(`Connected to ${window.StellarWallet.getProviderDisplayName(provider)}`, 'success');
                    announce(`Successfully connected to ${provider} wallet`);
                } else {
                    showToast(`Failed to connect: ${result.error}`, 'error');
                    announce(`Failed to connect to ${provider} wallet: ${result.error}`, 'assertive');
                    connectBtn.textContent = 'Connect Wallet';
                    connectBtn.disabled = false;
                }
            });
        });

        // Disconnect wallet
        if (disconnectBtn) {
            disconnectBtn.addEventListener('click', () => {
                window.StellarWallet.disconnectWallet();
                updateWalletUI(null, null);
                walletDropdown.hidden = true;
                walletMenuToggle.setAttribute('aria-expanded', 'false');
                showToast('Wallet disconnected', 'info');
                announce('Wallet disconnected');
            });
        }

        // Switch wallet
        if (switchWalletBtn) {
            switchWalletBtn.addEventListener('click', () => {
                walletDropdown.hidden = true;
                walletMenuToggle.setAttribute('aria-expanded', 'false');
                walletMenu.hidden = false;
                connectBtn.setAttribute('aria-expanded', 'true');
                announce('Wallet menu opened for switching');
            });
        }

        // Wallet menu toggle
        if (walletMenuToggle) {
            walletMenuToggle.addEventListener('click', () => {
                const isOpen = !walletDropdown.hidden;
                walletDropdown.hidden = isOpen;
                walletMenuToggle.setAttribute('aria-expanded', !isOpen);
            });
        }

        // Close menus when clicking outside
        document.addEventListener('click', (e) => {
            if (!e.target.closest('#wallet-container')) {
                walletMenu.hidden = true;
                walletDropdown.hidden = true;
                connectBtn.setAttribute('aria-expanded', 'false');
                walletMenuToggle.setAttribute('aria-expanded', 'false');
            }
        });

        // Listen for wallet events
        document.addEventListener('wallet:connected', (e) => {
            state.walletConnected = true;
            state.walletProvider = e.detail.provider;
            state.walletAddress = e.detail.address;
        });

        document.addEventListener('wallet:disconnected', () => {
            state.walletConnected = false;
            state.walletProvider = null;
            state.walletAddress = null;
        });
    }

    function updateWalletUI(address, provider) {
        const connectBtn = $('#wallet-connect-btn');
        const walletConnected = $('#wallet-connected');
        const walletProviderBadge = $('#wallet-provider-badge');
        const walletAddressDisplay = $('#wallet-address-display');

        if (address && provider) {
            connectBtn.hidden = true;
            walletConnected.hidden = false;
            
            const displayName = window.StellarWallet.getProviderDisplayName(provider);
            const formattedAddress = window.StellarWallet.formatAddress(address);
            
            walletProviderBadge.textContent = displayName;
            walletProviderBadge.className = `wallet-provider-badge ${provider}`;
            walletAddressDisplay.textContent = formattedAddress;
            walletAddressDisplay.title = address;
            
            state.walletConnected = true;
            state.walletProvider = provider;
            state.walletAddress = address;
        } else {
            connectBtn.hidden = false;
            walletConnected.hidden = true;
            connectBtn.textContent = 'Connect Wallet';
            connectBtn.disabled = false;
            
            state.walletConnected = false;
            state.walletProvider = null;
            state.walletAddress = null;
        }
    }

    function restoreWalletConnection() {
        if (window.StellarWallet && window.StellarWallet.loadWalletPreferences()) {
            const walletState = window.StellarWallet.getState();
            if (walletState.address && walletState.provider) {
                updateWalletUI(walletState.address, walletState.provider);
                announce(`Wallet restored: ${window.StellarWallet.getProviderDisplayName(walletState.provider)}`);
            }
        }
    }

    // ============================================
    // Reduced Motion Preference
    // ============================================
    function checkReducedMotion() {
        const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
        
        prefersReducedMotion.addEventListener('change', (e) => {
            if (e.matches) {
                document.documentElement.style.setProperty('--transition-fast', '0ms');
                document.documentElement.style.setProperty('--transition-normal', '0ms');
                announce('Reduced motion enabled');
            } else {
                document.documentElement.style.setProperty('--transition-fast', '150ms ease');
                document.documentElement.style.setProperty('--transition-normal', '250ms ease');
                announce('Reduced motion disabled');
            }
        });

        if (prefersReducedMotion.matches) {
            document.documentElement.style.setProperty('--transition-fast', '0ms');
            document.documentElement.style.setProperty('--transition-normal', '0ms');
        }
    }

    // ============================================
    // Initialization
    // ============================================
    async function init() {
        console.log('Initializing StellarEscrow Dashboard...');

        // Error boundary — must be first
        initErrorBoundary();

        // Performance monitoring
        observeWebVitals();
        initLazyRoutes();
        prefetchOnIdle();

        // Load preferences
        loadHighContrastPreference();
        loadSearchHistory();

        // Initialize UI components
        initMobileMenu();
        initHelpNav();
        initKeyboardNavigation();
        initForms();
        initSmoothScroll();
        initFocusManagement();
        checkReducedMotion();

        // Initialize wallet
        initWalletUI();
        restoreWalletConnection();

        // Initialize disputes
        if (window.DisputeManager) {
            window.DisputeManager.initDisputeUI();
        }

        // Initialize arbitrator dashboard
        if (window.ArbitratorDashboard) {
            window.ArbitratorDashboard.initArbitratorDashboard();
        }

        // Initialize notifications
        if (window.NotificationManager) {
            window.NotificationManager.initNotificationUI();
            window.NotificationManager.loadPreferences();
            window.NotificationManager.connectWebSocket();
        }

        // Initialize high contrast toggle
        const contrastToggle = $('#contrast-toggle');
        if (contrastToggle) {
            contrastToggle.addEventListener('click', toggleHighContrast);
        }

        // Initialize language switcher
        const langSelect = $('#lang-select');
        if (langSelect) {
            langSelect.value = getLocale();
            langSelect.addEventListener('change', (e) => {
                setLocale(e.target.value);
                announce(document.documentElement.lang === 'ar' ? 'تم تغيير اللغة' : 'Language changed');
            });
        }

        // Initialize PWA
        const swReg = await registerServiceWorker();
        initOfflineIndicator();

        // Install banner
        document.addEventListener('pwa:installable', () => {
            const banner = $('#install-banner');
            if (banner) banner.hidden = false;
        });
        $('#install-btn')?.addEventListener('click', async () => {
            const accepted = await promptInstall();
            if (accepted) $('#install-banner').hidden = true;
        });
        $('#install-dismiss')?.addEventListener('click', () => {
            $('#install-banner').hidden = true;
        });

        // Push notifications (subscribe after SW ready)
        if (swReg) {
            document.addEventListener('pwa:push-subscribe', async () => {
                await subscribePush(swReg);
            });
        }

        // Fetch initial data
        await fetchHealth();
        await fetchEvents();
        
        // Initialize help content
        renderFAQs();
        renderTutorials();

        // Connect WebSocket (will gracefully fail if server not running)
        // connectWebSocket();

        // Announce ready state
        announce('StellarEscrow Dashboard loaded and ready');
        console.log('Dashboard initialized');
    }

    // Run initialization when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

})();
