/**
 * StellarEscrow Dashboard - Accessible JavaScript Application
 * WCAG 2.1 AA Compliance Implementation
 */

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
        focusedElement: null
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
            const response = await fetch(`${CONFIG.apiBaseUrl}/health`);
            const data = await response.json();
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
            const response = await fetch(`${CONFIG.apiBaseUrl}/events?${queryParams}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            const data = await response.json();
            state.events = data;
            state.totalEvents = data.length;
            renderEventsTable(data);
            announce(`Loaded ${data.length} events`);
            return data;
        } catch (error) {
            console.error('Failed to fetch events:', error);
            showToast('Failed to load events', 'error');
            announce('Failed to load events', 'assertive');
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
    // Trade Detail View (Issue #31)
    // ============================================

    const STATUS_LABELS = {
        trade_created:   'Created',
        trade_funded:    'Funded',
        trade_completed: 'Completed',
        trade_confirmed: 'Confirmed',
        trade_disputed:  'Disputed',
        trade_cancelled: 'Cancelled',
        dispute_resolved:'Resolved',
    };

    const STATUS_CLASS = {
        trade_created:   'created',
        trade_funded:    'funded',
        trade_completed: 'completed',
        trade_confirmed: 'confirmed',
        trade_disputed:  'disputed',
        trade_cancelled: 'cancelled',
        dispute_resolved:'resolved',
    };

    const ACTION_LABELS = {
        Fund:           { label: 'Fund Trade',        cls: 'action-fund' },
        Complete:       { label: 'Mark Complete',     cls: 'action-complete' },
        ConfirmReceipt: { label: 'Confirm Receipt',   cls: 'action-confirm' },
        RaiseDispute:   { label: 'Raise Dispute',     cls: 'action-dispute' },
        Cancel:         { label: 'Cancel Trade',      cls: 'action-cancel' },
        ResolveDispute: { label: 'Resolve Dispute',   cls: 'action-resolve' },
    };

    async function fetchTradeDetail(tradeId) {
        try {
            const response = await fetch(`${CONFIG.apiBaseUrl}/trades/${encodeURIComponent(tradeId)}`);
            if (!response.ok) {
                if (response.status === 404) throw new Error(`Trade #${tradeId} not found`);
                throw new Error(`HTTP ${response.status}`);
            }
            return await response.json();
        } catch (error) {
            console.error('Failed to fetch trade detail:', error);
            throw error;
        }
    }

    function formatAmount(stroops) {
        if (stroops == null) return '—';
        return `${(stroops / 1e7).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 7 })} USDC`;
    }

    function truncateAddress(addr) {
        if (!addr || addr.length < 12) return addr || '—';
        return `${addr.slice(0, 6)}…${addr.slice(-6)}`;
    }

    function renderTradeDetail(data) {
        const panel = $('#trade-detail-panel');
        const empty = $('#trade-detail-empty');
        if (!panel || !empty) return;

        // Show panel, hide empty state
        panel.hidden = false;
        empty.hidden = true;

        // Basic info
        $('#td-trade-id').textContent = data.trade_id;

        const statusBadge = $('#td-status').querySelector('.status-badge') || document.createElement('span');
        statusBadge.className = `status-badge ${STATUS_CLASS[data.status] || ''}`;
        statusBadge.textContent = STATUS_LABELS[data.status] || data.status;
        $('#td-status').innerHTML = '';
        $('#td-status').appendChild(statusBadge);

        $('#td-amount').textContent = formatAmount(data.amount);
        $('#td-fee').textContent = data.fee != null ? formatAmount(data.fee) : '—';
        $('#td-seller-payout').textContent = data.seller_payout != null ? formatAmount(data.seller_payout) : '—';

        const sellerEl = $('#td-seller');
        sellerEl.textContent = truncateAddress(data.seller);
        sellerEl.title = data.seller || '';

        const buyerEl = $('#td-buyer');
        buyerEl.textContent = truncateAddress(data.buyer);
        buyerEl.title = data.buyer || '';

        const arbRow = $('#td-arbitrator-row');
        if (data.arbitrator) {
            arbRow.hidden = false;
            const arbEl = $('#td-arbitrator');
            arbEl.textContent = truncateAddress(data.arbitrator);
            arbEl.title = data.arbitrator;
        } else {
            arbRow.hidden = true;
        }

        $('#td-created-at').textContent = data.created_at ? formatTimestamp(data.created_at) : '—';
        $('#td-updated-at').textContent = data.updated_at ? formatTimestamp(data.updated_at) : '—';

        // Metadata
        const metaCard = $('#trade-metadata-card');
        const metaList = $('#trade-metadata-list');
        if (data.metadata && typeof data.metadata === 'object' && Object.keys(data.metadata).length > 0) {
            metaCard.hidden = false;
            metaList.innerHTML = Object.entries(data.metadata).map(([k, v]) => `
                <div class="trade-info-row">
                    <dt>${escapeHtml(k)}</dt>
                    <dd>${escapeHtml(String(v))}</dd>
                </div>
            `).join('');
        } else {
            metaCard.hidden = true;
        }

        // Timeline
        renderTimeline(data.timeline || [], data.status);

        // Transaction history
        renderTxHistory(data.transaction_history || []);

        // Actions (derived from status since viewer context isn't available in indexer)
        renderTradeActions(data);

        // Wire up share/export buttons
        const copyBtn = $('#copy-trade-link-btn');
        if (copyBtn) {
            copyBtn.onclick = () => {
                const url = `${window.location.origin}${window.location.pathname}#trade-detail?id=${data.trade_id}`;
                navigator.clipboard.writeText(url).then(() => {
                    showToast('Trade link copied to clipboard', 'success');
                    announce('Trade link copied to clipboard');
                }).catch(() => showToast('Failed to copy link', 'error'));
            };
        }

        const csvBtn = $('#export-trade-csv-btn');
        if (csvBtn) {
            csvBtn.onclick = () => exportTradeCSV(data);
        }

        const jsonBtn = $('#export-trade-json-btn');
        if (jsonBtn) {
            jsonBtn.onclick = () => exportTradeJSON(data);
        }

        announce(`Trade #${data.trade_id} loaded. Status: ${STATUS_LABELS[data.status] || data.status}`);
    }

    function renderTimeline(timeline, currentStatus) {
        const list = $('#trade-timeline');
        if (!list) return;

        if (timeline.length === 0) {
            list.innerHTML = '<li class="empty-state">No timeline data available</li>';
            return;
        }

        list.innerHTML = timeline.map((entry, i) => {
            const isLast = i === timeline.length - 1;
            const cls = isLast ? (STATUS_CLASS[entry.status] || 'active') : 'completed';
            const label = STATUS_LABELS[entry.status] || entry.status;
            const ts = entry.timestamp ? formatTimestamp(entry.timestamp) : '';
            const hash = entry.transaction_hash
                ? `<div class="timeline-hash" title="${escapeHtml(entry.transaction_hash)}">Tx: ${escapeHtml(entry.transaction_hash.slice(0, 16))}…</div>`
                : '';
            return `
                <li class="timeline-item ${cls}" aria-label="${escapeHtml(label)} at ledger ${entry.ledger}">
                    <div class="timeline-status">${escapeHtml(label)}</div>
                    <div class="timeline-meta">Ledger ${entry.ledger}${ts ? ` · ${ts}` : ''}</div>
                    ${hash}
                </li>
            `;
        }).join('');
    }

    function renderTxHistory(events) {
        const tbody = $('#tx-history-body');
        if (!tbody) return;

        if (events.length === 0) {
            tbody.innerHTML = '<tr><td colspan="4" class="empty-state">No transactions</td></tr>';
            return;
        }

        tbody.innerHTML = events.map(e => `
            <tr tabindex="0">
                <td>${e.ledger}</td>
                <td><span class="event-type">${escapeHtml(e.event_type)}</span></td>
                <td>${e.timestamp ? formatTimestamp(e.timestamp) : '—'}</td>
                <td class="address-cell" title="${escapeHtml(e.transaction_hash || '')}">${
                    e.transaction_hash ? escapeHtml(e.transaction_hash.slice(0, 16)) + '…' : '—'
                }</td>
            </tr>
        `).join('');
    }

    function renderTradeActions(data) {
        const container = $('#trade-actions-list');
        if (!container) return;

        // Derive available actions from status (viewer-agnostic display)
        const actionMap = {
            trade_created:   ['Fund', 'Cancel'],
            trade_funded:    ['Complete', 'RaiseDispute'],
            trade_completed: ['ConfirmReceipt', 'RaiseDispute'],
            trade_disputed:  ['ResolveDispute'],
            trade_confirmed: [],
            trade_cancelled: [],
            dispute_resolved:[],
        };

        const actions = actionMap[data.status] || [];
        if (actions.length === 0) {
            container.innerHTML = '<p class="empty-state">No actions available for this trade status</p>';
            return;
        }

        container.innerHTML = actions.map(action => {
            const { label, cls } = ACTION_LABELS[action] || { label: action, cls: '' };
            return `
                <button
                    type="button"
                    class="btn btn-secondary trade-action-btn ${cls}"
                    data-action="${escapeHtml(action)}"
                    aria-label="${escapeHtml(label)} for trade #${data.trade_id}"
                >
                    ${escapeHtml(label)}
                </button>
            `;
        }).join('');

        // Action click handler — shows informational toast (actual execution requires wallet)
        container.querySelectorAll('.trade-action-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const action = btn.dataset.action;
                showToast(`Action "${ACTION_LABELS[action]?.label || action}" requires wallet connection`, 'info');
                announce(`${ACTION_LABELS[action]?.label || action} action selected`);
            });
        });
    }

    function exportTradeCSV(data) {
        const rows = [
            ['trade_id', 'seller', 'buyer', 'amount', 'fee', 'seller_payout', 'status', 'created_at', 'updated_at'],
            [
                data.trade_id,
                data.seller,
                data.buyer,
                data.amount,
                data.fee ?? '',
                data.seller_payout ?? '',
                data.status,
                data.created_at ?? '',
                data.updated_at ?? '',
            ],
        ];
        const csv = rows.map(r => r.map(v => `"${String(v).replace(/"/g, '""')}"`).join(',')).join('\n');
        downloadFile(`trade_${data.trade_id}.csv`, csv, 'text/csv');
        announce(`Trade #${data.trade_id} exported as CSV`);
    }

    function exportTradeJSON(data) {
        const json = JSON.stringify(data, null, 2);
        downloadFile(`trade_${data.trade_id}.json`, json, 'application/json');
        announce(`Trade #${data.trade_id} exported as JSON`);
    }

    function downloadFile(filename, content, mimeType) {
        const blob = new Blob([content], { type: mimeType });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();
        URL.revokeObjectURL(url);
    }

    function initTradeDetailForm() {
        const form = $('#trade-lookup-form');
        if (!form) return;

        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            const input = $('#trade-detail-id');
            const tradeId = input?.value?.trim();
            if (!tradeId) return;

            const empty = $('#trade-detail-empty');
            const panel = $('#trade-detail-panel');
            if (empty) { empty.hidden = false; empty.querySelector('p').textContent = 'Loading…'; }
            if (panel) panel.hidden = true;

            try {
                const data = await fetchTradeDetail(tradeId);
                renderTradeDetail(data);
            } catch (err) {
                if (panel) panel.hidden = true;
                if (empty) {
                    empty.hidden = false;
                    empty.querySelector('p').textContent = err.message || 'Failed to load trade';
                }
                showToast(err.message || 'Failed to load trade', 'error');
                announce(err.message || 'Failed to load trade', 'assertive');
            }
        });

        // Support deep-link: #trade-detail?id=123
        const hash = window.location.hash;
        const match = hash.match(/[?&]id=(\d+)/);
        if (match) {
            const input = $('#trade-detail-id');
            if (input) {
                input.value = match[1];
                form.dispatchEvent(new Event('submit'));
            }
        }
    }

    // ============================================
    // Funding Interface (Issue #32)
    // ============================================

    const fundingState = {
        tradeId: null,
        trade: null,
        buyerAddress: null,
        currentStep: 1,
    };

    // Advance / retreat the step indicator UI
    function setFundingStep(step) {
        fundingState.currentStep = step;

        // Show/hide panels
        [1, 2, 3, 4].forEach(n => {
            const panel = $(`#funding-step-${n}`);
            if (panel) panel.hidden = (n !== step);
        });

        // Update step indicators
        [1, 2, 3, 4].forEach(n => {
            const ind = $(`#step-indicator-${n}`);
            if (!ind) return;
            ind.classList.remove('active', 'completed');
            if (n < step)  ind.classList.add('completed');
            if (n === step) ind.classList.add('active');
            ind.setAttribute('aria-current', n === step ? 'step' : 'false');
        });

        // Move focus to the new panel heading
        const heading = $(`#funding-step-${step}-heading`);
        if (heading) {
            heading.setAttribute('tabindex', '-1');
            heading.focus({ preventScroll: false });
        }

        announce(`Step ${step} of 4`);
    }

    // Validate a Stellar public key (G... 56 chars)
    function isValidStellarAddress(addr) {
        return /^G[A-Z2-7]{55}$/.test(addr.trim());
    }

    function showFieldError(errId, message) {
        const el = $(`#${errId}`);
        if (!el) return;
        el.textContent = message;
        el.hidden = false;
    }

    function clearFieldError(errId) {
        const el = $(`#${errId}`);
        if (!el) return;
        el.textContent = '';
        el.hidden = true;
    }

    function setButtonLoading(btn, loading) {
        if (!btn) return;
        if (loading) {
            btn.classList.add('btn-loading');
            btn.disabled = true;
            btn.setAttribute('aria-busy', 'true');
        } else {
            btn.classList.remove('btn-loading');
            btn.disabled = false;
            btn.removeAttribute('aria-busy');
        }
    }

    // Step 1 → 2: load trade and populate breakdown
    async function handleFundingLookup(e) {
        e.preventDefault();
        clearFieldError('funding-trade-id-err');
        clearFieldError('funding-buyer-address-err');

        const tradeIdInput = $('#funding-trade-id');
        const buyerInput   = $('#funding-buyer-address');
        const tradeId      = tradeIdInput?.value?.trim();
        const buyerAddress = buyerInput?.value?.trim();

        let valid = true;

        if (!tradeId || isNaN(tradeId) || Number(tradeId) < 1) {
            showFieldError('funding-trade-id-err', 'Please enter a valid trade ID.');
            tradeIdInput?.focus();
            valid = false;
        }

        if (!buyerAddress) {
            showFieldError('funding-buyer-address-err', 'Please enter your wallet address.');
            if (valid) buyerInput?.focus();
            valid = false;
        } else if (!isValidStellarAddress(buyerAddress)) {
            showFieldError('funding-buyer-address-err', 'Address must be a valid Stellar public key (starts with G, 56 characters).');
            if (valid) buyerInput?.focus();
            valid = false;
        }

        if (!valid) return;

        const btn = $('#funding-lookup-btn');
        setButtonLoading(btn, true);

        try {
            const trade = await fetchTradeDetail(tradeId);

            // Validate trade is fundable
            if (trade.status !== 'trade_created') {
                showFieldError('funding-trade-id-err',
                    `Trade #${tradeId} cannot be funded — current status: ${STATUS_LABELS[trade.status] || trade.status}.`);
                return;
            }

            // Validate buyer matches
            if (trade.buyer && trade.buyer.toLowerCase() !== buyerAddress.toLowerCase()) {
                showFieldError('funding-buyer-address-err',
                    'This address is not the buyer for this trade.');
                return;
            }

            fundingState.tradeId     = tradeId;
            fundingState.trade       = trade;
            fundingState.buyerAddress = buyerAddress;

            populateFundingBreakdown(trade, buyerAddress);
            setFundingStep(2);

        } catch (err) {
            showFieldError('funding-trade-id-err', err.message || 'Failed to load trade.');
        } finally {
            setButtonLoading(btn, false);
        }
    }

    function populateFundingBreakdown(trade, buyerAddress) {
        const amount  = trade.amount;
        const fee     = trade.fee ?? 0;
        const payout  = trade.seller_payout ?? (amount - fee);

        $('#fs-trade-id').textContent = trade.trade_id;
        $('#fs-seller').textContent   = truncateAddress(trade.seller);
        $('#fs-seller').title         = trade.seller || '';
        $('#fs-amount').textContent   = formatAmount(amount);
        $('#fs-fee').textContent      = fee > 0 ? formatAmount(fee) : 'None';
        $('#fs-total').textContent    = formatAmount(amount);   // buyer pays full amount
        $('#fs-payout').textContent   = formatAmount(payout);

        // Inline approval label
        const approvalInline = $('#approval-amount-inline');
        if (approvalInline) approvalInline.textContent = formatAmount(amount);

        // Reset approval checkbox
        const checkbox = $('#usdc-approval-checkbox');
        if (checkbox) checkbox.checked = false;
        const approveBtn = $('#funding-approve-btn');
        if (approveBtn) approveBtn.disabled = true;
    }

    // Step 2 → 3: populate confirmation card
    function handleApproveAndContinue() {
        clearFieldError('usdc-approval-err');

        const checkbox = $('#usdc-approval-checkbox');
        if (!checkbox?.checked) {
            showFieldError('usdc-approval-err', 'You must approve the USDC transfer to continue.');
            checkbox?.focus();
            return;
        }

        const trade = fundingState.trade;
        $('#fc-trade-id').textContent = trade.trade_id;
        $('#fc-buyer').textContent    = truncateAddress(fundingState.buyerAddress);
        $('#fc-buyer').title          = fundingState.buyerAddress;
        $('#fc-amount').textContent   = formatAmount(trade.amount);

        setFundingStep(3);
    }

    // Step 3 → 4: submit funding transaction
    async function handleFundingConfirm() {
        const btn = $('#funding-confirm-btn');
        setButtonLoading(btn, true);
        announce('Submitting funding transaction…', 'assertive');

        try {
            // Submit to indexer which proxies to the contract
            const response = await fetch(`${CONFIG.apiBaseUrl}/trades/${fundingState.tradeId}/fund`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ buyer: fundingState.buyerAddress }),
            });

            setFundingStep(4);

            if (response.ok) {
                showFundingSuccess(fundingState.trade);
            } else {
                const err = await response.json().catch(() => ({}));
                showFundingFailure(err.message || `Server error: ${response.status}`);
            }
        } catch (err) {
            setFundingStep(4);
            showFundingFailure(err.message || 'Network error — please try again.');
        } finally {
            setButtonLoading(btn, false);
        }
    }

    function showFundingSuccess(trade) {
        $('#funding-success').hidden  = false;
        $('#funding-failure').hidden  = true;
        $('#fr-trade-id').textContent = trade.trade_id;
        $('#fr-amount').textContent   = formatAmount(trade.amount);
        announce(`Trade #${trade.trade_id} funded successfully`, 'assertive');
    }

    function showFundingFailure(message) {
        $('#funding-success').hidden  = true;
        $('#funding-failure').hidden  = false;
        $('#funding-error-msg').textContent = message;
        announce(`Funding failed: ${message}`, 'assertive');
    }

    function resetFundingFlow() {
        fundingState.tradeId      = null;
        fundingState.trade        = null;
        fundingState.buyerAddress = null;

        // Clear inputs
        const tradeInput  = $('#funding-trade-id');
        const buyerInput  = $('#funding-buyer-address');
        if (tradeInput) tradeInput.value  = '';
        if (buyerInput) buyerInput.value  = '';

        clearFieldError('funding-trade-id-err');
        clearFieldError('funding-buyer-address-err');
        clearFieldError('usdc-approval-err');

        setFundingStep(1);
        $('#funding-trade-id')?.focus();
    }

    function initFundingInterface() {
        // Step 1 form submit
        const lookupForm = $('#funding-lookup-form');
        if (lookupForm) lookupForm.addEventListener('submit', handleFundingLookup);

        // Approval checkbox enables the approve button
        const checkbox   = $('#usdc-approval-checkbox');
        const approveBtn = $('#funding-approve-btn');
        if (checkbox && approveBtn) {
            checkbox.addEventListener('change', () => {
                approveBtn.disabled = !checkbox.checked;
                clearFieldError('usdc-approval-err');
            });
        }

        // Step 2 → 3
        if (approveBtn) approveBtn.addEventListener('click', handleApproveAndContinue);

        // Back buttons
        const backBtn  = $('#funding-back-btn');
        const backBtn2 = $('#funding-back-btn-2');
        if (backBtn)  backBtn.addEventListener('click',  () => setFundingStep(1));
        if (backBtn2) backBtn2.addEventListener('click', () => setFundingStep(2));

        // Step 3 confirm
        const confirmBtn = $('#funding-confirm-btn');
        if (confirmBtn) confirmBtn.addEventListener('click', handleFundingConfirm);

        // Result actions
        const newBtn   = $('#funding-new-btn');
        const retryBtn = $('#funding-retry-btn');
        if (newBtn)   newBtn.addEventListener('click',   resetFundingFlow);
        if (retryBtn) retryBtn.addEventListener('click', () => setFundingStep(3));

        // "View Trade Detail" button — jump to trade detail section
        const viewBtn = $('#funding-view-trade-btn');
        if (viewBtn) {
            viewBtn.addEventListener('click', () => {
                const idInput = $('#trade-detail-id');
                if (idInput && fundingState.tradeId) {
                    idInput.value = fundingState.tradeId;
                    const form = $('#trade-lookup-form');
                    if (form) form.dispatchEvent(new Event('submit'));
                }
                const section = $('#trade-detail');
                if (section) {
                    section.scrollIntoView({ behavior: 'smooth' });
                    section.setAttribute('tabindex', '-1');
                    section.focus({ preventScroll: true });
                }
            });
        }

        // Deep-link: #fund-trade?id=123
        const hash  = window.location.hash;
        const match = hash.match(/[?&]id=(\d+)/);
        if (match && window.location.hash.includes('fund-trade')) {
            const input = $('#funding-trade-id');
            if (input) input.value = match[1];
        }
    }

    // ============================================
    // Initialization
    // ============================================
    async function init() {
        console.log('Initializing StellarEscrow Dashboard...');

        // Load preferences
        loadHighContrastPreference();
        loadSearchHistory();

        // Initialize UI components
        initMobileMenu();
        initHelpNav();
        initKeyboardNavigation();
        initForms();
        initTradeDetailForm();
        initFundingInterface();
        initSmoothScroll();
        initFocusManagement();
        checkReducedMotion();

        // Initialize high contrast toggle
        const contrastToggle = $('#contrast-toggle');
        if (contrastToggle) {
            contrastToggle.addEventListener('click', toggleHighContrast);
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
